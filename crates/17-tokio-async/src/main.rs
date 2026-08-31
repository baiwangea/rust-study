//! Tokio 异步编程进阶示例。
//!
//! 覆盖：select!、join!/try_join!、Semaphore 限流、
//! CancellationToken 取消、三种通道（mpsc/broadcast/watch）、超时控制。

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Semaphore, broadcast, mpsc, watch};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<()> {
    println!("--- 1. select!：等待多个分支中先就绪的一个 ---");
    select_demo().await;

    println!("\n--- 2. join! / try_join!：并发等待多个 Future ---");
    join_demo().await;

    println!("\n--- 3. Semaphore：限制并发数 ---");
    semaphore_demo().await?;

    println!("\n--- 4. CancellationToken：任务取消 ---");
    cancellation_demo().await?;

    println!("\n--- 5. 三种通道：mpsc / broadcast / watch ---");
    channel_demo().await;

    println!("\n--- 6. timeout：超时控制 ---");
    timeout_demo().await;

    Ok(())
}

// --- 1. select! ---
async fn select_demo() {
    let fast = tokio::time::sleep(Duration::from_millis(50));
    let slow = tokio::time::sleep(Duration::from_millis(500));

    // 哪个分支先就绪就走哪个，另一个被丢弃（取消）
    tokio::select! {
        _ = fast => println!("fast 分支先完成"),
        _ = slow => println!("slow 分支先完成"),
    }
}

// --- 2. join! / try_join! ---
async fn fetch_user() -> Result<String> {
    tokio::time::sleep(Duration::from_millis(80)).await;
    Ok("user:alice".to_string())
}

async fn fetch_orders() -> Result<String> {
    tokio::time::sleep(Duration::from_millis(120)).await;
    Ok("orders:3".to_string())
}

async fn join_demo() {
    // join!：并发执行，全部完成后一起返回（耗时 ≈ 最慢的那个）
    let start = std::time::Instant::now();
    let (user, orders) = tokio::join!(fetch_user(), fetch_orders());
    println!("join! 并发结果: {:?}, {:?}，耗时 {:?}", user, orders, start.elapsed());

    // try_join!：任一分支出错立即返回错误（短路）
    let failing = async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Err::<String, anyhow::Error>(anyhow::anyhow!("模拟失败"))
    };
    let result = tokio::try_join!(fetch_user(), failing);
    println!("try_join! 短路结果: {:?}", result.err().map(|e| e.to_string()));
}

// --- 3. Semaphore ---
async fn semaphore_demo() -> Result<()> {
    // 许可证数量为 2：最多 2 个任务同时执行
    let semaphore = Arc::new(Semaphore::new(2));
    let mut handles = Vec::new();

    for i in 0..5 {
        let semaphore = Arc::clone(&semaphore);
        handles.push(tokio::spawn(async move {
            // acquire 在拿不到许可证时挂起，守卫被丢弃时自动归还
            let _permit = semaphore.acquire().await.unwrap();
            println!("任务 {} 开始执行（当前并发受限为 2）", i);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }));
    }

    for handle in handles {
        handle.await?;
    }
    println!("全部任务执行完毕");
    Ok(())
}

// --- 4. CancellationToken ---
async fn cancellation_demo() -> Result<()> {
    let token = CancellationToken::new();
    let child_token = token.clone();

    let worker = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = child_token.cancelled() => {
                    println!("worker 收到取消信号，清理资源后退出");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_millis(30)) => {
                    println!("worker 心跳...");
                }
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    token.cancel(); // 通知所有持有该 token（或其子 token）的任务
    worker.await?;
    Ok(())
}

// --- 5. 三种通道 ---
async fn channel_demo() {
    // mpsc：多生产者单消费者，有缓冲，满了背压
    let (tx, mut rx) = mpsc::channel::<i32>(8);
    tokio::spawn(async move {
        for i in 0..3 {
            tx.send(i).await.unwrap();
        }
    });
    while let Some(v) = rx.recv().await {
        println!("mpsc 收到: {}", v);
    }

    // broadcast：一个发送者，多个订阅者各自收到全部消息
    let (tx, mut rx1) = broadcast::channel::<&str>(4);
    let mut rx2 = tx.subscribe();
    tx.send("系统公告").unwrap();
    println!("broadcast 订阅者1: {}", rx1.recv().await.unwrap());
    println!("broadcast 订阅者2: {}", rx2.recv().await.unwrap());

    // watch：单值最新状态（适合配置热更新），接收端始终只看到最新值
    let (tx, mut rx) = watch::channel("v1.0.0");
    tx.send("v1.1.0").unwrap();
    rx.changed().await.unwrap(); // 等待值变化
    println!("watch 最新配置: {}", *rx.borrow_and_update());
}

// --- 6. timeout ---
async fn timeout_demo() {
    let slow_operation = async {
        tokio::time::sleep(Duration::from_millis(500)).await;
        "慢操作的结果"
    };

    // 超过时限返回 Err(Elapsed)，原 Future 被取消
    match tokio::time::timeout(Duration::from_millis(100), slow_operation).await {
        Ok(result) => println!("在时限内完成: {}", result),
        Err(_) => println!("操作超时，已放弃"),
    }
}
