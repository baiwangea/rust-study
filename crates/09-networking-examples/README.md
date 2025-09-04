# 网络编程实例 (networking-examples)

本项目演示 Rust 中两种不同层级的网络编程：

1.  **高级 HTTP 客户端**: 使用 `reqwest` 库从一个公共的 JSON API (jsonplaceholder.typicode.com) 获取数据，并将其反序列化为 Rust 结构体。这代表了现代应用中与外部服务进行通信的常见方式。

2.  **底层 TCP Socket**: 使用 `tokio::net` 创建一个异步的 TCP 回声服务器（Echo Server）和客户端。客户端发送消息，服务器接收后原样返回。这有助于理解更底层的网络协议交互。

## 如何运行

程序会先执行 HTTP 客户端示例，然后启动 TCP 服务器和客户端进行交互。

```bash
cargo run
```
