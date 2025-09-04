[中文](#chinese) | [English](#english)

---
<a id="chinese"></a>
# Rust 学习与实践 (Rust Study & Practice)

本项目是一个 Rust 学习工作区，旨在通过一系列独立的、可运行的实例项目，深入学习和实践 Rust 语言的各项核心功能和现代开发实践。

## 模块列表 (Module List)

本项目使用 Cargo Workspace 进行管理，所有独立的学习项目（Crates）都存放在 `crates/` 目录下。

- **`crates/01-log-examples`**: 日志操作实例 (log, tracing)。
- **`crates/02-redis-examples`**: Redis 客户端操作实例 (redis-rs)。
- **`crates/03-data-structures`**: 常用数据结构实现。
- **`crates/04-oop-concepts`**: 面向对象编程思想在 Rust 中的体现 (Struct, Enum, Trait)。
- **`crates/05-concurrency`**: 基础并发与多线程编程 (thread, channel, Mutex)。
- **`crates/06-web-api`**: 现代化 Web API 接口实战 (axum, JWT)。
- **`crates/07-solana-web3`**: Solana Web3 开发交互实例。
- **`crates/08-std-library-examples`**: Rust 标准库核心功能实践。
- **`crates/09-networking-examples`**: 网络编程 (HTTP Client & TCP Socket)。
- **`crates/10-delay-queue-examples`**: 异步延迟队列实现。

## 如何使用 (Usage)

### 运行单个项目 (Run a Single Crate)

进入具体的项目目录，使用 `cargo run`。

```bash
cd crates/01-log-examples
cargo run
```

### 检查或构建所有项目 (Check or Build All Crates)

在根目录下，可以一次性检查或构建所有项目。

```bash
# 检查所有项目是否存在编译错误
cargo check

# 构建所有项目
cargo build
```

---
<a id="english"></a>
# Rust Study & Practice

This project is a Rust learning workspace designed for in-depth study and practice of Rust's core features and modern development practices through a series of independent, runnable example projects.

## Module List

This project is managed using Cargo Workspace. All individual learning projects (crates) are located in the `crates/` directory.

- **`crates/01-log-examples`**: Examples for logging (log, tracing).
- **`crates/02-redis-examples`**: Examples for Redis client operations (redis-rs).
- **`crates/03-data-structures`**: Implementations of common data structures.
- **`crates/04-oop-concepts`**: Demonstrating Object-Oriented Programming concepts in Rust (Struct, Enum, Trait).
- **`crates/05-concurrency`**: Basic concurrency and multi-threading (thread, channel, Mutex).
- **`crates/06-web-api`**: Hands-on practice for modern Web API with `axum` and JWT.
- **`crates/07-solana-web3`**: Examples for Solana Web3 development interaction.
- **`crates/08-std-library-examples`**: Practice for core features of the Rust Standard Library.
- **`crates/09-networking-examples`**: Network programming (HTTP Client & TCP Socket).
- **`crates/10-delay-queue-examples`**: Implementation of an asynchronous delay queue.

## Usage

### Run a Single Crate

Navigate to a specific project directory and use `cargo run`.

```bash
cd crates/01-log-examples
cargo run
```

### Check or Build All Crates

You can check or build all projects at once from the root directory.

```bash
# Check all crates for compilation errors
cargo check

# Build all crates
cargo build
```
