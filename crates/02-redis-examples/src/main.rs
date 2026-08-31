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
    let client = redis::Client::open("redis://127.0.0.1/")?;
    // ConnectionManager 内部维护一条连接并自动重连，
    // 可 Clone 后在多个异步任务间安全共享
    let mut con = ConnectionManager::new(client).await?;
    println!("连接 Redis 成功 (ConnectionManager 自动重连模式)");

    string_demo(&mut con).await?;
    hash_demo(&mut con).await?;
    list_demo(&mut con).await?;
    set_demo(&mut con).await?;
    zset_demo(&mut con).await?;
    ttl_demo(&mut con).await?;
    pipeline_demo(&mut con).await?;

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
    let _: () = con.zadd_multiple(key, &[("alice", 100), ("bob", 200), ("carol", 150)]).await?;

    // ZRANGE ... WITHSCORES 返回扁平的 (member, score) 序列，可直接解析为元组数组
    let top: Vec<(String, i64)> = redis::cmd("ZRANGE")
        .arg(key)
        .arg(0)
        .arg(-1)
        .arg("WITHSCORES")
        .query_async(con)
        .await?;
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
    pipe.atomic()
        .cmd("DEL")
        .arg("demo:pipeline_counter")
        .incr("demo:pipeline_counter", 1)
        .incr("demo:pipeline_counter", 1)
        .incr("demo:pipeline_counter", 1);
    let results: (i64, i64, i64) = pipe.query_async(con).await?;
    println!("三次 INCR 的结果（忽略 DEL）: {:?}", results);
    Ok(())
}

async fn cleanup(con: &mut ConnectionManager) -> Result<()> {
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
