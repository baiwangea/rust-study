//! PostgreSQL 灵活便捷处理示例（SQLx + Postgres）。
//!
//! 与 `12-db-sqlx`（SQLite）的区别：
//! - 这里主打「灵活」——任意查询直接得到 JSON，不必为每个结果手写结构体；
//! - 提供泛型 CRUD 助手，表名 + 字段映射即可增删改查；
//! - 编译期零配置（不使用 `query!` 宏，不需要 `DATABASE_URL` 才能编译）。
//!
//! 前置条件：本地 PostgreSQL（见下方 README 说明）。运行：
//! ```bash
//! cargo run -p pg-flex
//! ```

mod db;
mod flex;

use anyhow::Result;
use flex::PqValue;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<()> {
    println!("连接 PostgreSQL ...");
    let pool = db::connect().await?;
    println!("连接成功，连接池已建立（最大 10 连接）");

    // 自动迁移：执行 ./migrations 下未应用过的 SQL（幂等）
    sqlx::migrate!("./migrations").run(&pool).await?;
    println!("数据库迁移完成（已确保 users 表存在）");

    // 1) 便捷插入：传字段映射即可，重复运行靠 ON CONFLICT 兜底
    flex::insert(
        &pool,
        "users",
        &[("username", PqValue::Text("alice".into())), ("balance", PqValue::Int(200))],
        Some("(username) DO UPDATE SET balance = EXCLUDED.balance"),
    )
    .await?;
    flex::insert(
        &pool,
        "users",
        &[("username", PqValue::Text("bob".into())), ("balance", PqValue::Int(100))],
        Some("(username) DO UPDATE SET balance = EXCLUDED.balance"),
    )
    .await?;
    flex::insert(
        &pool,
        "users",
        &[("username", PqValue::Text("carol".into())), ("balance", PqValue::Int(300))],
        Some("(username) DO UPDATE SET balance = EXCLUDED.balance"),
    )
    .await?;
    println!("\n[1] 已写入/更新 3 行用户（重复运行安全）");

    // 2) 任意 SQL → JSON：这是「灵活」的核心，无需定义结构体
    println!("\n[2] 灵活查询（余额 > 150 按余额倒序），结果直接是 JSON：");
    let rich = flex::fetch_json(
        &pool,
        "SELECT username, balance FROM users WHERE balance > $1 ORDER BY balance DESC",
        &[PqValue::Int(150)],
    )
    .await?;
    print_json(&rich);

    // 3) 泛型查询助手
    println!("\n[3] find_all 全表：");
    print_json(&flex::find_all(&pool, "users").await?);

    println!("\n[3b] find_where 按用户名查询：");
    print_json(
        &flex::find_where(
            &pool,
            "users",
            &[("username", PqValue::Text("alice".into()))],
        )
        .await?,
    );

    // 4) 更新助手
    println!("\n[4] 给 alice 加 50：");
    let n = flex::update_where(
        &pool,
        "users",
        &[("balance", PqValue::Int(250))],
        &[("username", PqValue::Text("alice".into()))],
    )
    .await?;
    println!("受影响 {} 行，更新后：", n);
    print_json(
        &flex::find_where(&pool, "users", &[("username", PqValue::Text("alice".into()))]).await?,
    );

    // 5) 删除助手
    println!("\n[5] 删除 carol：");
    let n = flex::delete_where(
        &pool,
        "users",
        &[("username", PqValue::Text("carol".into()))],
    )
    .await?;
    println!("删除 {} 行，剩余：", n);
    print_json(&flex::find_all(&pool, "users").await?);

    // 6) 事务：转账原子操作（alice -50, bob +50，中途失败自动回滚）
    println!("\n[6] 事务转账（alice -50 → bob +50，提交）：");
    transaction_transfer(&pool, "alice", "bob", 50).await?;
    print_json(&flex::find_all(&pool, "users").await?);

    println!("\nPostgreSQL 灵活示例执行完毕。");
    Ok(())
}

/// 漂亮打印 JSON 数组（失败时退回单行）。
fn print_json(rows: &[serde_json::Value]) {
    match serde_json::to_string_pretty(rows) {
        Ok(s) => println!("{}", s),
        Err(_) => println!("{:?}", rows),
    }
}

/// 事务转账演示：在事务内做两笔更新，体现「要么都成功，要么都回滚」。
async fn transaction_transfer(pool: &PgPool, from: &str, to: &str, amount: i64) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE users SET balance = balance - $1 WHERE username = $2")
        .bind(amount)
        .bind(from)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE users SET balance = balance + $1 WHERE username = $2")
        .bind(amount)
        .bind(to)
        .execute(&mut *tx)
        .await?;
    // 显式提交；若此处之前返回 Err，tx 被丢弃会自动回滚
    tx.commit().await?;
    println!("转账成功：{} -{}, {} +{}", from, amount, to, amount);
    Ok(())
}
