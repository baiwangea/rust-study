use std::sync::mpsc; // mpsc: multiple producer, single consumer
use std::thread;
use std::time::Duration;

fn main() {
    // 创建一个新的异步通道，tx 是发送端，rx 是接收端
    let (tx, rx) = mpsc::channel();

    println!("在主线程中，准备派生一个子线程...");

    // 派生一个子线程
    thread::spawn(move || {
        let vals = vec![
            String::from("hi"),
            String::from("from"),
            String::from("the"),
            String::from("thread"),
        ];

        for val in vals {
            println!("子线程: 正在发送 '{}'", val);
            // 使用发送端 tx 发送消息，move 关键字获取了 tx 的所有权
            tx.send(val).unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    });

    println!("主线程: 等待从子线程接收消息...");

    // 在主线程中，使用接收端 rx 接收消息
    // recv() 会阻塞主线程，直到有消息传来
    // 也可以使用 try_recv() 进行非阻塞接收
    // 当通道关闭时，迭代器会结束
    for received in rx {
        println!("主线程: 收到消息 '{}'", received);
    }

    println!("主线程: 通道已关闭，程序结束。");
}
