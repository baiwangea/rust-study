use log::{debug, error, info, trace, warn};

fn main() {
    // 初始化 env_logger，它会读取 RUST_LOG 环境变量
    env_logger::init();

    // 记录不同级别的日志
    // 使用 `cargo run` 默认看不到 trace 和 debug
    // 执行 `RUST_LOG=info cargo run` 可以看到 info, warn, error
    // 执行 `RUST_LOG=trace cargo run` 可以看到所有级别的日志
    info!("程序启动");

    let data = vec!["a", "b", "c"];
    trace!("这是一个详细的 trace 日志, data={:?}", data);
    debug!("这是一个用于调试的 debug 日志");

    warn!("这是一个警告信息！");
    error!("发生了一个错误！");

    info!("程序结束");
}
