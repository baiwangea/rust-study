//! 异步延迟队列实现。
//!
//! 设计要点：
//! - `BinaryHeap` + 自定义 `Ord` 反转 → 最小堆（最早到期的任务在堆顶）
//! - `Arc<Mutex>` 保证并发安全，`Notify` 高效唤醒等待中的 worker
//! - `Notify::enable()` 先注册监听再检查队列，避免丢失通知的竞态
//! - `CancellationToken` 支持优雅停止

use chrono::{DateTime, Utc};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::time::Duration;
use tokio::sync::{Mutex, Notify};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

// --- 1. 任务定义 ---
#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub payload: String,
    pub execute_at: DateTime<Utc>,
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Task {}

// 为了让 BinaryHeap 成为最小堆（执行时间最早的在堆顶），反转比较方向；
// 到期时间相同时按 id 决出次序，保证全序与 Eq/Ord 语义一致
impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .execute_at
            .cmp(&self.execute_at)
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// --- 2. 延迟队列结构体 ---
#[derive(Clone)]
pub struct DelayQueue {
    tasks: Arc<Mutex<BinaryHeap<Task>>>,
    notify: Arc<Notify>,
    cancel: CancellationToken,
    /// 已执行任务计数（便于测试与监控）
    executed: Arc<AtomicUsize>,
}

/// worker 每轮循环的下一步动作
enum NextStep {
    /// 堆顶任务已到期，立即执行
    Execute(Task),
    /// 堆顶任务未到期，等待指定时长
    Sleep(Duration),
    /// 队列为空，等待新任务通知
    Idle,
}

impl DelayQueue {
    /// 创建一个新的延迟队列并启动后台 worker
    pub fn new() -> Self {
        let queue = Self {
            tasks: Arc::new(Mutex::new(BinaryHeap::new())),
            notify: Arc::new(Notify::new()),
            cancel: CancellationToken::new(),
            executed: Arc::new(AtomicUsize::new(0)),
        };

        tokio::spawn(queue.clone().worker());
        queue
    }

    /// 添加一个新任务
    pub async fn add_task(&self, payload: String, delay: Duration) {
        let execute_at = Utc::now() + chrono::Duration::from_std(delay).unwrap();
        let task = Task {
            id: Uuid::new_v4(),
            payload,
            execute_at,
        };

        println!("添加新任务: id={}, delay={:?}", task.id, delay);

        self.tasks.lock().await.push(task);
        // 通知 worker：可能有新任务需要立即处理或需要重新计算等待时间
        self.notify.notify_one();
    }

    /// 已执行任务数
    pub fn executed_count(&self) -> usize {
        self.executed.load(AtomicOrdering::Relaxed)
    }

    /// 优雅停止：worker 会在当前等待结束后退出
    pub fn shutdown(&self) {
        self.cancel.cancel();
    }

    // 后台 worker 循环
    async fn worker(self) {
        loop {
            // 先注册通知监听（必须在检查队列之前），
            // 否则"检查队列为空"和"开始等待"之间到达的 notify 会被丢失
            let notified = self.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();

            let step = {
                let mut tasks = self.tasks.lock().await;
                match tasks.peek() {
                    Some(task) if task.execute_at <= Utc::now() => {
                        NextStep::Execute(tasks.pop().unwrap())
                    }
                    Some(task) => {
                        let delay = (task.execute_at - Utc::now()).to_std().unwrap();
                        NextStep::Sleep(delay)
                    }
                    None => NextStep::Idle,
                }
            };

            match step {
                NextStep::Execute(task) => {
                    println!(
                        "\n>>>>> 执行任务: id={}, payload='{}' <<<<<",
                        task.id, task.payload
                    );
                    self.executed.fetch_add(1, AtomicOrdering::Relaxed);
                }
                NextStep::Sleep(delay) => {
                    tokio::select! {
                        // 取消信号优先级最高，保证优雅停止
                        _ = self.cancel.cancelled() => break,
                        _ = tokio::time::sleep(delay) => {}
                        // 新任务到来：回到循环重新计算等待时间
                        _ = &mut notified => {}
                    }
                }
                NextStep::Idle => {
                    tokio::select! {
                        _ = self.cancel.cancelled() => break,
                        _ = &mut notified => {}
                    }
                }
            }
        }
        println!("延迟队列 worker 已优雅停止");
    }
}

impl Default for DelayQueue {
    fn default() -> Self {
        Self::new()
    }
}

// --- 3. 主函数演示 ---
#[tokio::main]
async fn main() {
    println!("--- 异步延迟队列示例 ---");
    let queue = DelayQueue::new();

    // 添加几个不同延迟的任务
    queue
        .add_task("发送欢迎邮件".to_string(), Duration::from_secs(3))
        .await;
    queue
        .add_task("取消未支付订单".to_string(), Duration::from_secs(1))
        .await;
    queue
        .add_task("清理临时文件".to_string(), Duration::from_secs(5))
        .await;

    println!("\n所有任务已添加，等待后台 worker 执行...");
    println!("预期执行顺序: 1s -> 3s -> 5s");

    // 等待所有任务执行完毕（轮询执行计数）
    while queue.executed_count() < 3 {
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    // 优雅停止 worker
    queue.shutdown();
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("示例结束，共执行 {} 个任务", queue.executed_count());
}

// --- 4. 单元测试 ---
#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(delay_ms: i64) -> Task {
        Task {
            id: Uuid::new_v4(),
            payload: String::new(),
            execute_at: Utc::now() + chrono::Duration::milliseconds(delay_ms),
        }
    }

    /// 最小堆性质：弹出顺序应等于到期时间升序
    #[test]
    fn test_min_heap_ordering() {
        let mut heap = BinaryHeap::new();
        heap.push(make_task(300));
        heap.push(make_task(100));
        heap.push(make_task(200));

        let order: Vec<i64> = (0..3)
            .map(|_| {
                let task = heap.pop().unwrap();
                (task.execute_at - Utc::now()).num_milliseconds()
            })
            .collect();

        assert!(order.windows(2).all(|w| w[0] <= w[1]), "弹出顺序应为到期时间升序: {:?}", order);
    }

    /// 集成行为：任务到期后被执行，且可以优雅停止
    #[tokio::test]
    async fn test_task_execution_and_shutdown() {
        let queue = DelayQueue::new();
        queue.add_task("t1".to_string(), Duration::from_millis(20)).await;
        queue.add_task("t2".to_string(), Duration::from_millis(40)).await;

        // 最多等待 2 秒，直到两个任务都执行完
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        while queue.executed_count() < 2 && tokio::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert_eq!(queue.executed_count(), 2, "两个任务都应被执行");

        // 优雅停止不应阻塞
        queue.shutdown();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
