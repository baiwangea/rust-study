//! 常用模块 Demo - config + tracing 示例
use anyhow::Result;
use config as cfg;
use serde::Deserialize;
use std::env;
use std::fs;
use std::sync::mpsc;
use std::thread;
use tracing::{debug, error, info};

#[derive(Debug, Deserialize)]
struct AppConfig {
    #[serde(default = "default_log_level")]
    log_level: String,
    #[serde(default = "default_port")]
    server_port: u16,
    #[serde(default = "default_db")]
    database_url: String,
}

fn default_log_level() -> String {
    "info".to_string()
}
fn default_port() -> u16 {
    8080
}
fn default_db() -> String {
    "sqlite://:memory:".to_string()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("env") => show_env(),
        Some("file") => file_demo(args.get(2).map(|s| s.as_str()).unwrap_or("demo.txt")),
        Some("thread") => thread_demo(),
        Some("config") => config_demo(),
        Some("help") | None => print_help(),
        Some(other) => {
            eprintln!("Unknown command: {}", other);
            print_help();
        }
    }
}

fn print_help() {
    println!("常用模块 Demo\n\nUsage:\n  common-demo env        # 显示环境变量 RUST_LOG\n  common-demo file [p]   # 写入并读取文件 (default: demo.txt)\n  common-demo thread     # 简单线程通信示例\n  common-demo config     # 配置与 tracing 示例（读取 Config.toml 与环境变量）\n  common-demo help       # 显示帮助");
}

fn show_env() {
    let key = "RUST_LOG";
    let val = env::var(key).unwrap_or_else(|_| "(not set)".to_string());
    println!("{} = {}", key, val);
}

fn file_demo(path: &str) {
    let content = "hello from common modules demo";
    if let Err(e) = fs::write(path, content) {
        eprintln!("Failed to write {}: {}", path, e);
        return;
    }
    match fs::read_to_string(path) {
        Ok(s) => println!("Read from {}: {}", path, s),
        Err(e) => eprintln!("Failed to read {}: {}", path, e),
    }
}

fn thread_demo() {
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let msg = "hello from worker".to_string();
        tx.send(msg).unwrap();
    });

    match rx.recv() {
        Ok(msg) => println!("Received: {}", msg),
        Err(e) => eprintln!("Channel error: {}", e),
    }

    handle.join().expect("worker panicked");
}

fn config_demo() {
    if let Err(e) = try_config_demo() {
        eprintln!("config demo failed: {}", e);
    }
}

fn try_config_demo() -> Result<()> {
    // 加载配置（Config.toml 可选）并允许环境变量覆盖（以 APP_ 前缀）
    let settings = cfg::Config::builder()
        .add_source(cfg::File::with_name("Config").required(false))
        .add_source(cfg::Environment::with_prefix("APP"))
        .build()?;

    let cfg: AppConfig = settings.try_deserialize()?;

    // 如果 RUST_LOG 环境变量存在，优先使用它
    let rust_log = env::var("RUST_LOG").ok();
    let env_filter = rust_log.unwrap_or_else(|| cfg.log_level.clone());

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(env_filter))
        .init();

    info!("Starting config demo");
    debug!("Loaded config: {:?}", cfg);

    // 演示读取项
    println!("effective log_level = {}", cfg.log_level);
    println!("server_port = {}", cfg.server_port);
    println!("database_url = {}", cfg.database_url);

    // 模拟连接数据库 / 发出日志
    info!("Pretend to connect to {}", cfg.database_url);
    error!("This is a sample error log for demonstration");

    Ok(())
}
