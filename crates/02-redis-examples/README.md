# Redis 操作实例 (redis-examples)

本项目演示如何在 Rust 中使用 `redis-rs` 库连接到 Redis 服务器，并执行一些基本的操作，如 SET/GET, HSET/HGET 等。

## 准备工作

请确保您本地或网络上有一个正在运行的 Redis 服务器。

## 如何运行

默认情况下，程序会尝试连接到 `redis://127.0.0.1/`。

```bash
cargo run
```
