//! Rust 标准库核心功能实践。
//!
//! 覆盖：String/&str、Option/Result、文件系统、迭代器与闭包、
//! 智能指针（Rc/RefCell/Arc）、时间与 Duration。

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn main() {
    println!("--- Rust 标准库实战 ---");
    string_examples();
    option_result_examples();
    fs_examples().unwrap(); // fs_examples 返回 Result，这里简单 unwrap
    iterator_and_closure_examples();
    smart_pointer_examples();
    time_examples();
}

// 1. String 与 &str 示例
fn string_examples() {
    println!("\n--- 1. String 与 &str ---");
    let mut s = String::from("hello");
    s.push_str(", world!"); // String 拥有数据，可以修改
    println!("动态字符串 (String): {}", s);

    // &str 是对字符串数据的借用视图（切片），不拥有数据
    let s_slice: &str = &s[0..5];
    println!("字符串切片 (&str): {}", s_slice);

    // format! 拼接（复杂场景推荐，避免多次分配）
    let joined = format!("{} + {}", s_slice, "rust");
    println!("format! 拼接: {}", joined);
}

// 2. Option<T> 与 Result<T, E> 示例
fn option_result_examples() {
    println!("\n--- 2. Option 与 Result ---");
    let mut scores = HashMap::new();
    scores.insert("Alice", 100);

    let alice_score = scores.get("Alice");
    let bob_score = scores.get("Bob");

    // match 显式处理两种情况
    match alice_score {
        Some(score) => println!("Alice 的分数是: {}", score),
        None => println!("找不到 Alice 的分数"),
    }

    // 组合子链式处理，比 if let 更函数式
    let bob_info = bob_score
        .map(|score| format!("Bob 的分数是 {}", score))
        .unwrap_or_else(|| "找不到 Bob 的分数".to_string());
    println!("{}", bob_info);

    // Result：`?` 操作符在函数间传播错误
    let parsed = "42".parse::<i32>();
    match parsed {
        Ok(num) => println!("解析成功: {}", num),
        Err(e) => println!("解析失败: {}", e),
    }
}

// 3. 文件系统 (std::fs) 与路径 (std::path) 示例
fn fs_examples() -> std::io::Result<()> {
    println!("\n--- 3. 文件系统与路径 ---");
    let path = Path::new("./temp_rust_std_example.txt");

    let mut file = File::create(path)?;
    file.write_all(b"Hello, Rust file system!")?;
    println!("成功写入内容到: {:?}", path);

    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    println!("读取文件内容: '{}'", contents);

    fs::remove_file(path)?;
    println!("成功删除文件: {:?}", path);

    // Path 常用操作
    let p = Path::new("/var/log/app/main.log");
    println!("父目录: {:?}, 文件名: {:?}", p.parent(), p.file_name());
    Ok(())
}

// 4. 迭代器 (Iterator) 与闭包 (Closure) 示例
fn iterator_and_closure_examples() {
    println!("\n--- 4. 迭代器与闭包 ---");
    let numbers = vec![1, 2, 3, 4, 5];

    // 迭代器是惰性的：只有被消费时才真正执行
    let processed: Vec<i32> = numbers
        .iter()
        .map(|&n| n * 2)
        .filter(|&n| n > 5)
        .collect();
    println!("map + filter: {:?}", processed);

    let sum: i32 = numbers.iter().fold(0, |acc, &x| acc + x);
    println!("fold 求和: {}", sum);

    // 三种闭包特性：
    // Fn：只读捕获（可多次调用）
    // FnMut：可变捕获（每次调用可能修改捕获的变量）
    // FnOnce：消费捕获的变量（只能调用一次）
    let threshold = 3;
    let is_big = |n: &i32| *n > threshold; // Fn: 只读借用 threshold
    println!("大于 {} 的元素: {:?}", threshold, numbers.iter().filter(|n| is_big(n)).collect::<Vec<_>>());

    let mut counter = 0;
    let mut count = || {
        counter += 1; // FnMut: 可变借用 counter
        counter
    };
    count();
    println!("FnMut 闭包计数: {}", count());

    let owned = String::from("被消费的数据");
    let consume = move || println!("FnOnce 闭包拥有: {}", owned); // FnOnce: 获得所有权
    consume();
    // consume() 不能再调用，`owned` 已被移动
}

// 5. 智能指针示例
fn smart_pointer_examples() {
    println!("\n--- 5. 智能指针 ---");

    // Rc：单线程共享所有权（引用计数）
    let shared = Rc::new(String::from("共享数据"));
    let clone1 = Rc::clone(&shared);
    println!("Rc 内容: {}, 当前引用数: {}", clone1, Rc::strong_count(&shared));

    // RefCell：内部可变性——即使是不可变绑定，运行时也能修改内容
    let cache: RefCell<Vec<i32>> = RefCell::new(Vec::new());
    cache.borrow_mut().push(42); // 运行时借用检查
    println!("RefCell 内容: {:?}", cache.borrow());

    // Arc：Rc 的线程安全版本（Atomic Reference Counting）
    let arc_data = Arc::new(vec![1, 2, 3]);
    let thread_arc = Arc::clone(&arc_data);
    let handle = std::thread::spawn(move || {
        println!("子线程读取 Arc: {:?}", thread_arc);
    });
    handle.join().unwrap();
    println!("Arc 跨线程共享成功");
}

// 6. 时间处理示例
fn time_examples() {
    println!("\n--- 6. 时间与 Duration ---");
    let start = Instant::now();
    std::thread::sleep(Duration::from_millis(50));
    println!("程序运行耗时: {:?}", start.elapsed());

    // Duration 支持算术运算
    let total = Duration::from_secs(1) + Duration::from_millis(500);
    println!("1s + 500ms = {:?}", total);
}
