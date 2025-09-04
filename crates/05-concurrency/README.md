# 并发编程实例 (concurrency)

本项目演示 Rust 的核心并发特性。

- **线程**: 使用 `std::thread::spawn` 创建新线程。
- **通道 (Channels)**: 使用 `std::sync::mpsc`（多生产者，单消费者）在线程间安全地传递消息。

## 如何运行

```bash
cargo run
```
