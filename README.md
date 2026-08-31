[中文](#chinese) | [English](#english)

---
<a id="chinese"></a>
# Rust 学习与实践 (Rust Study & Practice)

本项目是一个 Rust 学习工作区，通过一系列独立的、可运行的实例项目，深入学习和实践 Rust 语言的核心功能与后端工程实践。使用 Cargo Workspace 管理，统一 edition 2024 与公共依赖版本。

## 模块列表 (Module List)

所有学习项目（Crates）都存放在 `crates/` 目录下：

### 语言基础
- **`03-data-structures`**: 常用数据结构（Vec/HashMap/VecDeque/BTree/BinaryHeap + 手写链表）。
- **`04-oop-concepts`**: 面向对象思想（Trait、trait object、静态 vs 动态分发、类型状态模式）。
- **`05-concurrency`**: 并发编程（mpsc 通道、Arc/Mutex、RwLock、原子类型、作用域线程）。
- **`08-std-library-examples`**: 标准库核心功能（String、Option/Result、文件 IO、迭代器、智能指针）。
- **`13-error-handling`**: 错误处理模式（thiserror 库层错误 + anyhow 应用层上下文）。
- **`14-testing`**: 测试实践（单元测试、集成测试、属性测试）。

### 异步与网络
- **`09-networking-examples`**: 网络编程（reqwest HTTP 客户端、tokio TCP Socket）。
- **`10-delay-queue-examples`**: 异步延迟队列（最小堆 + Notify + CancellationToken 优雅停止）。
- **`17-tokio-async`**: Tokio 异步进阶（select!/join!、信号量限流、任务取消、三种通道、超时）。

### 后端工程
- **`01-log-examples`**: 日志（log facade + tracing 结构化日志与 span）。
- **`02-redis-examples`**: Redis 异步客户端（五大数据结构、TTL、Pipeline 事务）。需本地 Redis。
- **`06-web-api`**: Web API 实战（axum 0.8、JWT 认证、统一错误响应、中间件、优雅关闭）。
- **`11-common-modules`**: 常用模块综合（config 配置加载、tracing、env/file/thread 子命令）。
- **`12-db-sqlx`**: 数据库实战（SQLx + SQLite：连接池、迁移、参数绑定、事务转账）。
- **`15-cli-tools`**: CLI 工具（clap 4 derive：子命令、类型化参数、JSON 持久化）。
- **`16-grpc-tonic`**: gRPC 服务（tonic：proto 定义、服务端实现、客户端调用与状态码处理）。需本机 `protoc`。

### 区块链
- **`07-solana-web3`**: Solana Web3 开发交互（Devnet 空投、查询余额）。需网络访问。

## 如何使用 (Usage)

### 运行单个项目

```bash
# 通过 -p 指定包名（推荐，无需切目录）
cargo run -p web-api
cargo run -p tokio-async

# 或进入目录运行
cd crates/01-log-examples && cargo run
```

### 检查、测试与构建

```bash
cargo check --workspace      # 检查全部编译错误
cargo clippy --workspace     # 静态检查
cargo test --workspace       # 运行全部测试
cargo build --workspace      # 构建全部
```

### 外部依赖说明

| 模块 | 前置条件 |
|------|----------|
| `02-redis-examples` | 本地启动 Redis：`docker run --rm -p 6379:6379 redis:7` |
| `07-solana-web3` | 可访问 Solana Devnet |
| `09-networking-examples` | 可访问 jsonplaceholder.typicode.com |
| `12-db-sqlx` | 无（自动创建 SQLite 文件 `study.db`） |
| `16-grpc-tonic` | 本机安装 `protoc`；先启动 `cargo run --bin grpc-server` 再运行 `grpc-client` |

---
<a id="english"></a>
# Rust Study & Practice

A Rust learning workspace: a series of independent, runnable example projects covering core language features and backend engineering practices. Managed as a Cargo Workspace with unified edition 2024 and shared dependency versions.

## Module List

All crates live under `crates/`:

### Language Fundamentals
- **`03-data-structures`**: Common data structures (Vec/HashMap/VecDeque/BTree/BinaryHeap + a hand-written linked list).
- **`04-oop-concepts`**: OOP in Rust (traits, trait objects, static vs dynamic dispatch, typestate pattern).
- **`05-concurrency`**: Concurrency (mpsc channels, Arc/Mutex, RwLock, atomics, scoped threads).
- **`08-std-library-examples`**: Standard library essentials (String, Option/Result, file IO, iterators, smart pointers).
- **`13-error-handling`**: Error handling patterns (thiserror for libraries + anyhow for applications).
- **`14-testing`**: Testing practices (unit tests, integration tests, property-based tests).

### Async & Networking
- **`09-networking-examples`**: Networking (reqwest HTTP client, tokio TCP sockets).
- **`10-delay-queue-examples`**: Async delay queue (min-heap + Notify + CancellationToken graceful shutdown).
- **`17-tokio-async`**: Advanced Tokio (select!/join!, semaphore limiting, cancellation, channel comparison, timeouts).

### Backend Engineering
- **`01-log-examples`**: Logging (log facade + tracing structured events and spans).
- **`02-redis-examples`**: Async Redis client (five data structures, TTL, pipeline transactions). Requires local Redis.
- **`06-web-api`**: Web API (axum 0.8, JWT auth, unified error responses, middleware, graceful shutdown).
- **`11-common-modules`**: Common modules (config loading, tracing, env/file/thread subcommands).
- **`12-db-sqlx`**: Database (SQLx + SQLite: connection pool, migrations, parameter binding, transactional transfer).
- **`15-cli-tools`**: CLI tooling (clap 4 derive: subcommands, typed arguments, JSON persistence).
- **`16-grpc-tonic`**: gRPC (tonic: proto definition, server implementation, client calls with status-code handling). Requires `protoc`.

### Blockchain
- **`07-solana-web3`**: Solana Web3 interaction (Devnet airdrop, balance queries). Requires network access.

## Usage

### Run a single crate

```bash
cargo run -p web-api
cargo run -p tokio-async
# or: cd crates/01-log-examples && cargo run
```

### Check, test, build

```bash
cargo check --workspace
cargo clippy --workspace
cargo test --workspace
cargo build --workspace
```

### External dependencies

| Crate | Requirement |
|-------|-------------|
| `02-redis-examples` | Local Redis: `docker run --rm -p 6379:6379 redis:7` |
| `07-solana-web3` | Access to Solana Devnet |
| `09-networking-examples` | Access to jsonplaceholder.typicode.com |
| `12-db-sqlx` | None (auto-creates SQLite file `study.db`) |
| `16-grpc-tonic` | `protoc` installed; start `cargo run --bin grpc-server` before `grpc-client` |
