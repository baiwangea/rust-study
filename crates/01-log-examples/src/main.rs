//! 日志示例：传统 `log` facade 与 `tracing` 结构化日志对比。
//!
//! 运行方式：
//! - `cargo run`                （默认显示 info 及以上）
//! - `RUST_LOG=debug cargo run` （显示 debug 及以上）
//! - `RUST_LOG=trace cargo run` （显示全部级别）

use tracing::{debug, info, instrument, warn};

fn main() {
    // 一次性初始化全局订阅器，从 RUST_LOG 环境变量读取过滤规则。
    // `tracing-subscriber` 内置了对 `log` crate 事件的兼容桥接，
    // 因此下面两种风格的日志都会被同一个订阅器捕获。
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // --- 1. 传统风格：log facade（适合库代码，保持实现无关） ---
    log::info!("程序启动 (经由 log facade 发出)");
    log::debug!("这是一个默认不可见的 debug 日志");
    log::warn!("这是一个警告信息！");
    log::error!("发生了一个错误！");

    // --- 2. 结构化风格：tracing（支持结构化字段） ---
    info!(module = "log-examples", version = "0.1.0", "程序启动 (tracing 事件)");
    debug!("RUST_LOG=debug 时才能看到这条调试日志");

    // --- 3. span 上下文：把一组事件关联到同一个逻辑操作 ---
    // `#[instrument]` 会为函数自动创建 span，并把参数记录进去
    handle_order(42);
    process_payment(42, 99.5);

    info!("程序结束");
}

#[instrument]
fn handle_order(order_id: u32) {
    info!("开始处理订单");
    debug!("校验订单明细");
    // 在 span 内部发出的事件会自动携带 order_id 上下文
    warn!("订单中包含库存紧张的商品");
}

#[instrument]
fn process_payment(order_id: u32, amount: f64) {
    info!("发起扣款");
    // 也可以手动为单条事件附加结构化字段
    info!(channel = "alipay", "支付网关响应成功");
}
