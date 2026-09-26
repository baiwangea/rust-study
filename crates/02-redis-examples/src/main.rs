//! 异步 Redis 客户端示例（redis-rs 1.x + tokio）。
//!
//! 前置条件：本地启动一个 Redis 服务，例如：
//! ```bash
//! docker run --rm -p 6379:6379 redis:7
//! ```

use anyhow::Result;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    // Redis 连接 URL 格式：redis://[用户名:密码@]主机[:端口][/数据库编号]
    // 示例：
    // - "redis://127.0.0.1/" 或 "redis://127.0.0.1/0" -> 连接到 0 号库（默认）
    // - "redis://127.0.0.1/1" -> 连接到 1 号库
    // - "redis://:password@127.0.0.1/2" -> 带密码连接到 2 号库
    // - "redis://user:password@127.0.0.1:6379/3" -> 完整格式（Redis 6+ ACL）
    let client: redis::Client = redis::Client::open("redis://127.0.0.1/3")?;
    
    // ConnectionManager 内部维护一条连接并自动重连，
    // 可 Clone 后在多个异步任务间安全共享
    let mut con: ConnectionManager = ConnectionManager::new(client).await?;
    println!("连接 Redis 成功 (ConnectionManager 自动重连模式，使用 0 号数据库)");

    // 【重要】先清理上次运行可能残留的测试数据
    // 避免旧数据（特别是 ZSet 中的整数分数）干扰本次示例的类型推导
    cleanup(&mut con).await?;

    string_demo(&mut con).await?;
    hash_demo(&mut con).await?;
    list_demo(&mut con).await?;
    set_demo(&mut con).await?;
    zset_demo(&mut con).await?;
    ttl_demo(&mut con).await?;
    pipeline_demo(&mut con).await?;
    
    // 演示动态切换数据库
    database_switch_demo(&mut con).await?;

    // 清理本次示例创建的 key
    cleanup(&mut con).await?;
    println!("\nRedis 示例执行完毕。");
    Ok(())
}

/// String：SET/GET/INCR 与过期时间
async fn string_demo(con: &mut ConnectionManager) -> Result<()> {
    println!("\n--- String (SET/GET/INCR) ---");
    let _: () = con.set("demo:str", "hello world").await?;
    let value: String = con.get("demo:str").await?;
    println!("GET 'demo:str': {}", value);

    // redis 1.x 中 INCR 需要显式传入增量
    let _: i64 = con.incr("demo:counter", 1).await?;
    let count: i64 = con.incr("demo:counter", 1).await?;
    println!("INCR 'demo:counter' x2 => {}", count);
    Ok(())
}

/// Hash：批量写入与一次性读取
async fn hash_demo(con: &mut ConnectionManager) -> Result<()> {
    println!("\n--- Hash (HSET/HGETALL) ---");
    let key = "demo:hash";
    let _: () = con
        .hset_multiple(key, &[("field1", "value1"), ("field2", "value2")])
        .await?;

    let all: HashMap<String, String> = con.hgetall(key).await?;
    println!("HGETALL '{}': {:?}", key, all);

    let field1: String = con.hget(key, "field1").await?;
    println!("HGET 'field1': {}", field1);
    Ok(())
}

/// List：RPUSH/LRANGE/LPOP
async fn list_demo(con: &mut ConnectionManager) -> Result<()> {
    println!("\n--- List (RPUSH/LRANGE/LPOP) ---");
    let key = "demo:list";
    let _: () = con.rpush(key, &["a", "b", "c"]).await?;
    let items: Vec<String> = con.lrange(key, 0, -1).await?;
    println!("LRANGE '{}': {:?}", key, items);

    // redis 1.x 中 LPOP 第二参数表示弹出个数（None = 弹出单个元素）
    let head: String = con.lpop(key, None).await?;
    println!("LPOP => '{}'", head);
    Ok(())
}

/// Set：SADD/SMEMBERS/SISMEMBER
async fn set_demo(con: &mut ConnectionManager) -> Result<()> {
    println!("\n--- Set (SADD/SMEMBERS/SISMEMBER) ---");
    let key = "demo:set";
    let _: () = con.sadd(key, &["rust", "go", "python"]).await?;
    let members: Vec<String> = con.smembers(key).await?;
    println!("SMEMBERS '{}': {:?}", key, members);

    let has_rust: bool = con.sismember(key, "rust").await?;
    println!("SISMEMBER 'rust' => {}", has_rust);
    Ok(())
}

/// ZSet：ZADD + 按分数排序读取
async fn zset_demo(con: &mut ConnectionManager) -> Result<()> {
    println!("\n--- ZSet (ZADD/ZRANGE WITHSCORES) ---");
    let key = "demo:leaderboard";
    
    // redis-rs 1.x 中，ZSet 的分数必须使用 f64 浮点数类型
    // 注意：虽然整数分数在 Redis 中有效，但 redis-rs 在解析 WITHSCORES 响应时
    // 会将分数统一按浮点数处理，因此写入时也应使用浮点数以保持类型一致
    let _: () = con.zadd(key, "alice", 100.0).await?;
    let _: () = con.zadd(key, "bob", 200.0).await?;
    let _: () = con.zadd(key, "carol", 150.0).await?;

    // zrange_withscores 是 AsyncCommands trait 提供的便捷方法
    // 直接返回 Vec<(String, f64)> 格式，无需手动解析 Redis 响应
    // 参数：key, start(0=第一个), stop(-1=最后一个)
    let top: Vec<(String, f64)> = con.zrange_withscores(key, 0, -1).await?;
    println!("排行榜（按分数升序）: {:?}", top);
    Ok(())
}

/// TTL：写入带过期时间的 key 并查询剩余生存时间
async fn ttl_demo(con: &mut ConnectionManager) -> Result<()> {
    println!("\n--- TTL (SET EX / TTL) ---");
    let _: () = con.set_ex("demo:temp", "60 秒后消失", 60).await?;
    let ttl: i64 = con.ttl("demo:temp").await?;
    println!("'demo:temp' 剩余生存时间: {} 秒", ttl);
    Ok(())
}

/// Pipeline：把多条命令打包发送，`atomic()` 等价于 MULTI/EXEC 事务
async fn pipeline_demo(con: &mut ConnectionManager) -> Result<()> {
    println!("\n--- Pipeline (atomic = MULTI/EXEC) ---");
    let mut pipe = redis::pipe();
    pipe.atomic()  // 开启事务模式（MULTI/EXEC），保证原子性
        .cmd("DEL")
        .arg("demo:pipeline_counter")
        .ignore()  // 关键：忽略 DEL 的返回值，不计入 query_async 的结果元组
                   // 如果不调用 ignore()，DEL 会返回删除的 key 数量（通常是 0 或 1）
                   // 这会导致返回值变成 4 元素数组而非 3 元素，无法解构为 (i64, i64, i64)
        .incr("demo:pipeline_counter", 1)  // 第 1 次自增，返回 1
        .incr("demo:pipeline_counter", 1)  // 第 2 次自增，返回 2
        .incr("demo:pipeline_counter", 1); // 第 3 次自增，返回 3
    
    // 只接收未被 ignore 的三个 INCR 命令的返回值
    let results: (i64, i64, i64) = pipe.query_async(con).await?;
    println!("三次 INCR 的结果: {:?}", results);  // 输出: (1, 2, 3)
    Ok(())
}

async fn cleanup(con: &mut ConnectionManager) -> Result<()> {
    // 使用原始 cmd 批量删除多个 key，返回删除数量（此处忽略）
    let _: () = redis::cmd("DEL")
        .arg("demo:str")
        .arg("demo:counter")
        .arg("demo:hash")
        .arg("demo:list")
        .arg("demo:set")
        .arg("demo:leaderboard")
        .arg("demo:temp")
        .arg("demo:pipeline_counter")
        .query_async(con)
        .await?;
    Ok(())
}
