use std::collections::HashMap;

fn main() {
    println!("--- Vec (动态数组) 示例 ---");
    let mut numbers = vec![1, 2, 3];
    numbers.push(4);
    println!("Vec: {:?}", numbers);
    println!("第一个元素: {}", numbers[0]);

    // 遍历 Vec
    for num in &numbers {
        println!("遍历元素: {}", num);
    }

    println!("\n--- HashMap (哈希图) 示例 ---");
    let mut scores = HashMap::new();
    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Yellow"), 50);

    let team_name = String::from("Blue");
    // 使用 get 返回 Option<&V>，更安全
    let score = scores.get(&team_name).copied().unwrap_or(0);
    println!("{} 队的分数: {}", team_name, score);

    // 遍历 HashMap
    for (key, value) in &scores {
        println!("{}: {}", key, value);
    }
}
