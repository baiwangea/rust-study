//! 并发编程示例：线程、通道与共享状态。
//!
//! 覆盖：mpsc 通道、Arc<Mutex>、RwLock、原子类型、thread::scope。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc; // mpsc: multiple producer, single consumer
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

fn main() {
    println!("--- 1. mpsc 通道：线程间消息传递 ---");
    channel_demo();

    println!("\n--- 2. Arc<Mutex>：多线程共享可变状态 ---");
    arc_mutex_demo();

    println!("\n--- 3. RwLock：读多写少场景 ---");
    rwlock_demo();

    println!("\n--- 4. Atomic：无锁计数 ---");
    atomic_demo();

    println!("\n--- 5. thread::scope：借用外部数据的作用域线程 ---");
    scoped_thread_demo();
}

fn channel_demo() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for val in ["hi", "from", "the", "thread"] {
            tx.send(val).unwrap(); // move 捕获了 tx 的所有权
            thread::sleep(Duration::from_millis(100));
        }
    });

    // 通道关闭（所有发送端被丢弃）时，迭代器自动结束
    for received in rx {
        println!("主线程收到: '{}'", received);
    }
}

fn arc_mutex_demo() {
    // Arc 提供线程安全的引用计数，Mutex 保护内部数据
    let counter = Arc::new(Mutex::new(0u64));
    let mut handles = Vec::new();

    for i in 0..5 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                // lock() 返回 RAII 守卫，离开作用域自动解锁
                *counter.lock().unwrap() += 1;
            }
            println!("线程 {} 完成累加", i);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
    println!("5 个线程各累加 1000 次，最终结果: {}", *counter.lock().unwrap());
}

fn rwlock_demo() {
    let config = Arc::new(RwLock::new(vec!["初始配置"]));
    let mut handles = Vec::new();

    // 多个读者可以同时持有读锁
    for i in 0..3 {
        let config = Arc::clone(&config);
        handles.push(thread::spawn(move || {
            let items = config.read().unwrap();
            println!("读者 {} 看到: {:?}", i, *items);
        }));
    }

    // 写锁是独占的
    {
        let mut items = config.write().unwrap();
        items.push("新增配置");
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn atomic_demo() {
    let total = Arc::new(AtomicU64::new(0));
    let mut handles = Vec::new();

    for _ in 0..5 {
        let total = Arc::clone(&total);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                // 无锁原子操作，比 Mutex 更轻量，但只适合简单数据
                total.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
    println!("原子计数结果: {}", total.load(Ordering::Relaxed));
}

fn scoped_thread_demo() {
    let mut data = vec![1, 2, 3];

    // scope 保证所有线程在结束前不会被提前析构借用目标，
    // 因此闭包可以直接借用栈上的数据，无需 move / Arc
    thread::scope(|s| {
        s.spawn(|| {
            println!("作用域线程读取: {:?}", data);
        });
        s.spawn(|| {
            println!("作用域线程也能看到长度: {}", data.len());
        });
    });

    // scope 结束后，可以安全地继续使用 data
    data.push(4);
    println!("scope 结束后继续使用: {:?}", data);
}
