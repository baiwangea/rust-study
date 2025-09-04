use serde::Deserialize;
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

// 用于反序列化 JSON 响应的结构体
#[derive(Deserialize, Debug)]
struct Post {
    id: u32,
    title: String,
    body: String,
    #[serde(rename = "userId")]
    user_id: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Rust 网络编程实战 ---");

    // 1. 运行 HTTP 客户端示例
    http_client_example().await?;

    // 2. 运行 TCP Socket 示例
    tcp_socket_example().await?;

    Ok(())
}

// --- 示例 1: 高级 HTTP 客户端 ---
async fn http_client_example() -> Result<(), Box<dyn Error>> {
    println!("\n--- 1. HTTP 客户端 (reqwest) ---");
    let url = "https://jsonplaceholder.typicode.com/posts/1";
    println!("正在从 {} 获取数据...", url);

    // 发送 GET 请求并等待响应
    let response = reqwest::get(url).await?;

    // 检查请求是否成功
    if response.status().is_success() {
        // 将 JSON 响应体反序列化为 Post 结构体
        let post: Post = response.json().await?;
        println!("成功获取并解析 Post:");
        println!("{:#?}", post);
    } else {
        println!("请求失败，状态码: {}", response.status());
    }

    Ok(())
}

// --- 示例 2: 底层 TCP Socket ---
async fn tcp_socket_example() -> Result<(), Box<dyn Error>> {
    println!("\n--- 2. TCP Socket (tokio::net) ---");
    let addr = "127.0.0.1:8080";

    // 在一个独立的异步任务中启动服务器
    let server = tokio::spawn(async move {
        let listener = TcpListener::bind(addr).await.unwrap();
        println!("[服务器] 正在监听 {}", addr);

        // 等待一个客户端连接
        let (mut socket, client_addr) = listener.accept().await.unwrap();
        println!("[服务器] 接受来自 {} 的连接", client_addr);

        let mut buf = [0; 1024];
        // 读取客户端数据
        let n = socket.read(&mut buf).await.unwrap();
        println!("[服务器] 收到 {} 字节数据，正在回显...", n);

        // 将数据原样写回
        socket.write_all(&buf[0..n]).await.unwrap();
    });

    // 给服务器一点时间来启动
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 在另一个异步任务中启动客户端
    let client = tokio::spawn(async move {
        let mut stream = TcpStream::connect(addr).await.unwrap();
        println!("[客户端] 成功连接到 {}", addr);
        let msg = b"Hello, TCP Server!";

        // 发送消息
        stream.write_all(msg).await.unwrap();
        println!("[客户端] 发送消息: '{}'", String::from_utf8_lossy(msg));

        // 读取服务器的回显
        let mut buf = [0; 1024];
        let n = stream.read(&mut buf).await.unwrap();
        println!("[客户端] 收到回显: '{}'", String::from_utf8_lossy(&buf[..n]));
    });

    // 等待服务器和客户端任务完成
    server.await?;
    client.await?;

    println!("\nTCP 示例执行完毕。");
    Ok(())
}
