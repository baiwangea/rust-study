//! SQLx 数据库实战（SQLite + 异步连接池）。
//!
//! 演示内容：
//! - 连接池（SqlitePool）与自动迁移（sqlx::migrate!）
//! - 参数绑定防注入、`query_as` 映射到结构体、动态行访问（Row）
//! - 事务：转账成功与余额不足回滚两个场景
//!
//! 说明：`query!` 系列编译期检查宏需要 `DATABASE_URL` 环境变量（或离线模式），
//! 本示例为保持零配置可运行，使用运行时绑定写法。

use anyhow::{Result, anyhow};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{FromRow, Row, SqlitePool};
use std::str::FromStr;

#[derive(Debug, FromRow)]
struct User {
    #[allow(dead_code)]
    id: i64,
    username: String,
    balance: i64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 连接池配置：数据库文件不存在时自动创建
    let options = SqliteConnectOptions::from_str("sqlite://study.db")?
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;
    println!("SQLite 连接池已创建（最大 5 连接，文件: study.db）");

    // 运行迁移：自动执行 ./migrations 下未应用过的 SQL
    sqlx::migrate!("./migrations").run(&pool).await?;
    println!("数据库迁移完成");

    seed_users(&pool).await?;
    query_demo(&pool).await?;
    transaction_demo(&pool, "alice", "bob", 50).await?;
    transaction_demo(&pool, "bob", "alice", 100_000).await?;

    println!("\n数据库示例执行完毕。");
    Ok(())
}

/// 初始化演示数据（INSERT OR REPLACE 保证可重复运行）
async fn seed_users(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO users (id, username, balance) VALUES (?1, ?2, ?3)",
    )
    .bind(1)
    .bind("alice")
    .bind(200_i64)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT OR REPLACE INTO users (id, username, balance) VALUES (?1, ?2, ?3)",
    )
    .bind(2)
    .bind("bob")
    .bind(100_i64)
    .execute(pool)
    .await?;

    println!("\n已写入演示数据: alice=200, bob=100");
    Ok(())
}

/// 查询演示：query_as 映射结构体 + Row 动态访问
async fn query_demo(pool: &SqlitePool) -> Result<()> {
    println!("\n--- 查询示例 ---");

    // query_as：自动把行映射为 User 结构体
    let users: Vec<User> = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY balance DESC")
        .fetch_all(pool)
        .await?;
    for user in &users {
        println!("query_as => {}", user);
    }

    // Row：按列名动态取值，适合聚合查询等不便定义结构体的场景
    let row = sqlx::query("SELECT COUNT(*) AS total, SUM(balance) AS sum FROM users")
        .fetch_one(pool)
        .await?;
    let total: i64 = row.get("total");
    let sum: i64 = row.get("sum");
    println!("Row 动态访问 => 共 {} 人，总余额 {}", total, sum);
    Ok(())
}

/// 事务演示：from -> to 转账；余额不足时回滚，保证原子性
async fn transaction_demo(pool: &SqlitePool, from: &str, to: &str, amount: i64) -> Result<()> {
    println!("\n--- 事务转账: {} -> {} 金额 {} ---", from, to, amount);

    let result = transfer(pool, from, to, amount).await;
    match result {
        Ok(()) => println!("转账成功"),
        Err(e) => println!("转账失败（已回滚）: {}", e),
    }

    let balances: Vec<User> = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY id")
        .fetch_all(pool)
        .await?;
    for user in balances {
        println!("转账后 => {}", user);
    }
    Ok(())
}

async fn transfer(pool: &SqlitePool, from: &str, to: &str, amount: i64) -> Result<()> {
    // 开启事务：`?` 提前返回时 tx 被丢弃会自动回滚
    let mut tx = pool.begin().await?;

    // 扣款并校验余额
    let result = sqlx::query("UPDATE users SET balance = balance - ?1 WHERE username = ?2 AND balance >= ?1")
        .bind(amount)
        .bind(from)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 0 {
        return Err(anyhow!("账户 {} 不存在或余额不足", from));
    }

    // 入账
    let result = sqlx::query("UPDATE users SET balance = balance + ?1 WHERE username = ?2")
        .bind(amount)
        .bind(to)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 0 {
        return Err(anyhow!("收款账户 {} 不存在", to));
    }

    // 显式提交；省略此行则自动回滚
    tx.commit().await?;
    Ok(())
}

impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "user={} balance={}", self.username, self.balance)
    }
}
