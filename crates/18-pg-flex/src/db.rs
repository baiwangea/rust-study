//! PostgreSQL 连接池与配置（灵活便捷的基础层）。
//!
//! 设计要点：
//! - 连接串走环境变量 `DATABASE_URL`，缺省指向本地 `study` 库，方便本地即开即用。
//! - 使用 [`PgPoolOptions`] 建立带健康检查的连接池，避免每次请求新建连接。
//! - 连接失败给出可读提示（而不是一堆底层报错），并指向 README 的本地启动步骤。

use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;

/// 读取连接串：环境变量优先，缺省本地 study 库。
///
/// 自定义连接可直接 `export DATABASE_URL=postgres://user:pass@host:5432/dbname`。
pub fn database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/study".to_string()
    })
}

/// 建立连接池（最大 10 连接）。
///
/// 失败时通过 [`Context`] 附带可读提示，方便排查「本地 PG 没起 / 库不存在」。
pub async fn connect() -> Result<PgPool> {
    let url = database_url();
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&url)
        .await
        .with_context(|| {
            format!(
                "无法连接 PostgreSQL：{url}\n→ 请先启动本地 PG（见 README 的 18-pg-flex 章节）并确保 `study` 库已创建。\n→ 快速起一个：docker run --rm -e POSTGRES_PASSWORD=postgres -p 5432:5432 postgres:16\n→ 建库：docker exec -i $(docker ps -q -f ancestor=postgres:16) psql -U postgres -c 'CREATE DATABASE study'"
            )
        })?;
    Ok(pool)
}
