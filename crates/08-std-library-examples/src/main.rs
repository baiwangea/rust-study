use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

// 主函数，程序的入口
fn main() {
    println!("--- Rust 标准库实战 ---");
    string_examples();
    option_result_examples();
    fs_examples().unwrap(); // fs_examples 返回 Result，这里简单 unwrap
    iterator_and_closure_examples();
}

// 1. String 与 &str 示例
fn string_examples() {
    println!("\n--- 1. String 与 &str ---");
    // 创建一个 String
    let mut s = String::from("hello");
    s.push_str(", world!"); // 修改 String
    println!("动态字符串 (String): {}", s);

    // &str 是对字符串数据的引用（切片）
    let s_slice: &str = &s[0..5];
    println!("字符串切片 (&str): {}", s_slice);
}

// 2. Option<T> 与 Result<T, E> 示例
fn option_result_examples() {
    println!("\n--- 2. Option 与 Result ---");
    // Option 用于处理可能为空的值
    let mut scores = HashMap::new();
    scores.insert("Alice", 100);

    // get 返回一个 Option<&i32>
    let alice_score: Option<&i32> = scores.get("Alice");
    let bob_score: Option<&i32> = scores.get("Bob");

    // 使用 match 处理 Option
    match alice_score {
        Some(score) => println!("Alice 的分数是: {}", score),
        None => println!("找不到 Alice 的分数"),
    }

    // 使用 if let 更简洁地处理 Some 的情况
    if let Some(score) = bob_score {
        println!("Bob 的分数是: {}", score);
    } else {
        println!("找不到 Bob 的分数");
    }

    // Result 用于处理可能失败的操作
    let result: Result<i32, &str> = "42".parse();
    match result {
        Ok(num) => println!("解析成功: {}", num),
        Err(e) => println!("解析失败: {}", e),
    }
}

// 3. 文件系统 (std::fs) 与路径 (std::path) 示例
fn fs_examples() -> std::io::Result<()> {
    println!("\n--- 3. 文件系统与路径 ---");
    let path = Path::new("./temp_rust_std_example.txt");

    // 写入文件
    let mut file = File::create(&path)?;
    file.write_all(b"Hello, Rust file system!")?;
    println!("成功写入内容到: {:?}", path);

    // 读取文件
    let mut file = File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    println!("读取文件内容: '{}'", contents);

    // 删除文件
    fs::remove_file(&path)?;
    println!("成功删除文件: {:?}", path);

    Ok(())
}

// 4. 迭代器 (Iterator) 与闭包 (Closure) 示例
fn iterator_and_closure_examples() {
    println!("\n--- 4. 迭代器与闭包 ---");
    let numbers = vec![1, 2, 3, 4, 5];

    // 闭包：一个可以捕获其环境的匿名函数
    let add_one = |x| x + 1;
    println!("闭包调用: add_one(5) = {}", add_one(5));

    // 迭代器与 map, filter, fold
    let processed_numbers: Vec<i32> = numbers
        .iter() // 创建迭代器
        .map(|&n| n * 2) // 使用 map 和闭包将每个元素乘以 2
        .filter(|&n| n > 5) // 使用 filter 和闭包保留大于 5 的元素
        .collect(); // 收集结果到一个新的 Vec

    println!("处理后的数组 (map, filter): {:?}", processed_numbers);

    let sum: i32 = numbers.iter().fold(0, |acc, &x| acc + x);
    println!("使用 fold 计算数组总和: {}", sum);
}
