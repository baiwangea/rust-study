//! 网络编程示例：HTTP 客户端（reqwest）与 TCP Socket（tokio）。
//!
//! 运行需要网络访问（HTTP 示例请求 jsonplaceholder）。

use anyhow::Result;
use serde::Deserialize;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// 用于反序列化 JSON 响应的结构体
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Post {
    id: u32,
    title: String,
    body: String,
    #[serde(rename = "userId")]
    user_id: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("--- Rust 网络编程实战 ---");

    http_client_example().await?;
    tcp_socket_example().await?;

    Ok(())
}

// --- 示例 1: HTTP 客户端 ---
async fn http_client_example() -> Result<()> {
    println!("\n--- 1. HTTP 客户端 (reqwest) ---");

    // 生产环境推荐复用同一个 Client：内部维护连接池，避免每次请求都建连
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10)) // 整体请求超时
        .connect_timeout(Duration::from_secs(5)) // 建连超时
        .build()?;

    let url = "https://jsonplaceholder.typicode.com/posts/1";
    println!("正在从 {} 获取数据...", url);

    let response = client.get(url).send().await?;

    // 显式处理状态码，而不是假设成功
    if response.status().is_success() {
        let post: Post = response.json().await?;
        println!("成功获取并解析 Post:");
        println!("{:#?}", post);
    } else {
        println!("请求失败，状态码: {}", response.status());
    }

    // 错误处理示例：访问不存在的资源会返回 404 而不是 Err
    let not_found = client
        .get("https://jsonplaceholder.typicode.com/posts/999999")
        .send()
        .await?;
    println!("访问不存在的资源，状态码: {} (is_success={})",
        not_found.status(),
        not_found.status().is_success());

    Ok(())
}

// --- 示例 2: TCP Socket（完整读写回显） ---
async fn tcp_socket_example() -> Result<()> {
    println!("\n--- 2. TCP Socket (tokio::net) ---");
    let addr = "127.0.0.1:8080";

    // 在独立的异步任务中启动回显服务器
    let server = tokio::spawn(async move {
        let listener = TcpListener::bind(addr).await?;
        println!("[服务器] 正在监听 {}", addr);

        let (mut socket, client_addr) = listener.accept().await?;
        println!("[服务器] 接受来自 {} 的连接", client_addr);

        let mut buf = [0u8; 1024];
        // 循环读取直到客户端断开（read 返回 0 表示对端关闭）
        loop {
            let n = socket.read(&mut buf).await?;
            if n == 0 {
                println!("[服务器] 客户端断开连接");
                break;
            }
            println!("[服务器] 收到 {} 字节，正在回显...", n);
            socket.write_all(&buf[0..n]).await?;
        }
        Ok::<(), anyhow::Error>(())
    });

    // 给服务器一点启动时间
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 客户端：发送两条消息并读取回显
    let client = tokio::spawn(async move {
        let mut stream = TcpStream::connect(addr).await?;
        println!("[客户端] 成功连接到 {}", addr);

        for msg in ["Hello, TCP Server!", "第二条消息"] {
            stream.write_all(msg.as_bytes()).await?;
            println!("[客户端] 发送: '{}'", msg);

            let mut buf = [0u8; 1024];
            let n = stream.read(&mut buf).await?;
            println!("[客户端] 收到回显: '{}'", String::from_utf8_lossy(&buf[..n]));
        }
        Ok::<(), anyhow::Error>(())
        // stream 在此被丢弃，服务器收到连接关闭
    });

    client.await??;
    server.await??;

    println!("\nTCP 示例执行完毕。");
    Ok(())
}
