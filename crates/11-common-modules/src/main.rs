//! 常用模块 Demo
use std::env;
use std::fs;
use std::sync::mpsc;
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("env") => show_env(),
        Some("file") => file_demo(args.get(2).map(|s| s.as_str()).unwrap_or("demo.txt")),
        Some("thread") => thread_demo(),
        Some("help") | None => print_help(),
        Some(other) => {
            eprintln!("Unknown command: {}", other);
            print_help();
        }
    }
}

fn print_help() {
    println!("常用模块 Demo\n\nUsage:\n  common-demo env        # 显示环境变量 RUST_LOG\n  common-demo file [p]   # 写入并读取文件 (default: demo.txt)\n  common-demo thread     # 简单线程通信示例\n  common-demo help       # 显示帮助");
}

fn show_env() {
    let key = "RUST_LOG";
    let val = env::var(key).unwrap_or_else(|_| "(not set)".to_string());
    println!("{} = {}", key, val);
}

fn file_demo(path: &str) {
    let content = "hello from common modules demo";
    if let Err(e) = fs::write(path, content) {
        eprintln!("Failed to write {}: {}", path, e);
        return;
    }
    match fs::read_to_string(path) {
        Ok(s) => println!("Read from {}: {}", path, s),
        Err(e) => eprintln!("Failed to read {}: {}", path, e),
    }
}

fn thread_demo() {
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let msg = "hello from worker".to_string();
        tx.send(msg).unwrap();
    });

    match rx.recv() {
        Ok(msg) => println!("Received: {}", msg),
        Err(e) => eprintln!("Channel error: {}", e),
    }

    handle.join().expect("worker panicked");
}
