use chrono::{DateTime, Utc};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Notify};
use uuid::Uuid;

// --- 1. 任务定义 ---
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Task {
    pub id: Uuid,
    pub payload: String,
    pub execute_at: DateTime<Utc>,
}

// 为了让 BinaryHeap 成为一个最小堆（执行时间最早的在堆顶），我们需要反转 `Ord` 的实现逻辑
impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        // 注意：other.cmp(self) 来实现最小堆
        other.execute_at.cmp(&self.execute_at)
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
    // 使用 Arc<Mutex<>> 来实现并发安全
    tasks: Arc<Mutex<BinaryHeap<Task>>>,
    // 使用 Notify 来高效地唤醒等待的 worker
    notify: Arc<Notify>,
}

impl DelayQueue {
    // 创建一个新的延迟队列并启动后台 worker
    pub fn new() -> Self {
        let queue = Self {
            tasks: Arc::new(Mutex::new(BinaryHeap::new())),
            notify: Arc::new(Notify::new()),
        };

        // 派生一个后台任务来处理队列
        tokio::spawn(queue.clone().worker());

        queue
    }

    // 添加一个新任务
    pub async fn add_task(&self, payload: String, delay: Duration) {
        let execute_at = Utc::now() + chrono::Duration::from_std(delay).unwrap();
        let task = Task {
            id: Uuid::new_v4(),
            payload,
            execute_at,
        };

        println!("添加新任务: id={}, delay={:?}", task.id, delay);

        let mut tasks = self.tasks.lock().await;
        tasks.push(task);

        // 通知 worker，可能有新任务需要立即处理或需要重新计算等待时间
        self.notify.notify_one();
    }

    // 后台 worker 循环
    async fn worker(self) {
        loop {
            let mut tasks = self.tasks.lock().await;

            if let Some(task) = tasks.peek() {
                let now = Utc::now();
                if task.execute_at <= now {
                    // 时间到了，处理任务
                    let task = tasks.pop().unwrap(); // 此时可以安全地 unwrap
                    println!("\n>>>>> 执行任务: id={}, payload='{}' <<<<<", task.id, task.payload);
                    // 释放锁，继续下一次循环
                    drop(tasks);
                    continue;
                }

                // 如果任务没到期，计算需要睡眠的时间
                let sleep_duration = (task.execute_at - now).to_std().unwrap();
                let notified = self.notify.clone();
                
                // 释放锁，然后等待
                drop(tasks);

                // 使用 select! 等待，要么是睡眠时间到，要么是被新任务通知唤醒
                tokio::select! {
                    _ = tokio::time::sleep(sleep_duration) => {
                        // 睡眠结束，进入下一个循环检查任务
                    }
                    _ = notified.notified() => {
                        // 被唤醒，立即进入下一个循环重新检查
                    }
                }
            } else {
                // 队列为空，等待通知
                let notified = self.notify.clone();
                drop(tasks); // 等待前必须释放锁
                notified.notified().await;
            }
        }
    }
}

// --- 3. 主函数，用于演示 ---
#[tokio::main]
async fn main() {
    println!("--- 异步延迟队列示例 ---");
    let queue = DelayQueue::new();

    // 添加几个不同延迟的任务
    queue.add_task("发送欢迎邮件".to_string(), Duration::from_secs(5)).await;
    queue.add_task("取消未支付订单".to_string(), Duration::from_secs(2)).await;
    queue.add_task("清理临时文件".to_string(), Duration::from_secs(8)).await;

    println!("\n所有任务已添加，等待后台 worker 执行...");
    println!("预期执行顺序: 2s -> 5s -> 8s");

    // 等待足够长的时间让所有任务被处理
    tokio::time::sleep(Duration::from_secs(10)).await;
}
