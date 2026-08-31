//! gRPC 客户端：调用 Greeter 服务，演示正常调用与错误状态码处理。
//!
//! 前置：先启动服务端 `cargo run --bin grpc-server`。

use grpc_tonic::hello::{HelloRequest, greeter_client::GreeterClient};
use tonic::Code;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = GreeterClient::connect("http://127.0.0.1:50051").await?;
    println!("已连接到 gRPC 服务");

    // --- 1. 正常调用 ---
    let response = client
        .say_hello(HelloRequest {
            name: "Rust".to_string(),
        })
        .await?;
    println!("服务端返回: {}", response.into_inner().message);

    // --- 2. 触发服务端校验错误：匹配 gRPC 状态码 ---
    let result = client
        .say_hello(HelloRequest {
            name: String::new(),
        })
        .await;
    match result {
        Err(status) if status.code() == Code::InvalidArgument => {
            println!("捕获到 InvalidArgument: {}", status.message());
        }
        Err(status) => println!("其他错误: {:?}", status),
        Ok(_) => println!("意外的成功响应"),
    }

    Ok(())
}
