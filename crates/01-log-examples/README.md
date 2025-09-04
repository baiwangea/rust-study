# 日志操作实例 (log-examples)

本项目演示如何在 Rust 中使用 `log` 和 `env_logger` 这两个流行的库来记录应用程序日志。

## 如何运行

你可以通过设置 `RUST_LOG` 环境变量来控制日志级别。可选的级别有 `error`, `warn`, `info`, `debug`, `trace`。

```bash
# 只显示 info 级别及以上的日志
export RUST_LOG=info
cargo run

# 显示所有级别的日志
export RUST_LOG=trace
cargo run
```
