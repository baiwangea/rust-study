//! 灵活便捷层：把任意查询结果变成 JSON，并提供泛型 CRUD。
//!
//! 为什么需要这一层？
//! - SQLx 的 `query_as!` 需要为每个查询手写结构体，且依赖编译期 `DATABASE_URL` 校验。
//! - 在很多「自定义处理 / 临时查询 / 内部工具」场景里，我们更想要 PHP 那种
//!   「关联数组」式的灵活结果：拿到一行就能按字段名取值，无需先定义类型。
//!
//! 本模块提供：
//! - [`PqValue`]：覆盖 Postgres 常见列类型的「灵活值」，未知类型自动降级为文本。
//! - [`row_to_json`] / [`fetch_json`]：任意 SQL → `Vec<serde_json::Value>`。
//! - [`insert`] / [`find_all`] / [`find_where`] / [`update_where`] / [`delete_where`]：
//!   表名 + 字段映射即可完成增删改查，省去样板 SQL。

use anyhow::{anyhow, Result};
use serde_json::{Map, Value};
use sqlx::postgres::Postgres;
use sqlx::query::Query;
use sqlx::{Column, Row, TypeInfo, ValueRef, Database, PgPool};

/// Postgres 查询类型别名：A 固定为 Postgres 的参数类型，避免手写冗长泛型。
type PgQuery<'q> = Query<'q, Postgres, <Postgres as Database>::Arguments>;

/// 统一的「灵活值」类型。
///
/// 对应 Postgres 常见列类型；遇到不认识的类型降级为 [`PqValue::Other`]，
/// 保证「任何查询都能拿到结果」而不是 panic。
#[derive(Debug, Clone)]
pub enum PqValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Json(Value),
    Bytes(Vec<u8>),
    Other(String),
}

impl PqValue {
    /// 转为 `serde_json::Value`，方便直接打印 / 走 API / 二次处理。
    pub fn to_json(&self) -> Value {
        match self {
            PqValue::Null => Value::Null,
            PqValue::Bool(b) => Value::Bool(*b),
            PqValue::Int(i) => Value::Number((*i).into()),
            PqValue::Float(f) => {
                serde_json::Number::from_f64(*f).map(Value::Number).unwrap_or(Value::Null)
            }
            PqValue::Text(s) => Value::String(s.clone()),
            PqValue::Json(j) => j.clone(),
            PqValue::Bytes(b) => Value::String(format!("{:?}", b)),
            PqValue::Other(s) => Value::String(s.clone()),
        }
    }
}

impl std::fmt::Display for PqValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PqValue::Null => write!(f, "NULL"),
            PqValue::Bool(b) => write!(f, "{}", b),
            PqValue::Int(i) => write!(f, "{}", i),
            PqValue::Float(x) => write!(f, "{}", x),
            PqValue::Text(s) => write!(f, "{}", s),
            PqValue::Json(j) => write!(f, "{}", j),
            PqValue::Bytes(b) => write!(f, "{:?}", b),
            PqValue::Other(s) => write!(f, "{}", s),
        }
    }
}

/// 按列类型把某一列解码成 [`PqValue`]。
///
/// `type_info().name()` 返回的是 Postgres 内部类型名（如 `INT4` / `TEXT` / `TIMESTAMPTZ`）。
/// 这里覆盖常用类型；`NUMERIC` 故意转字符串以避免浮点精度丢失。
fn decode_column(row: &sqlx::postgres::PgRow, ordinal: usize, type_name: &str) -> Result<PqValue> {
    let raw = row.try_get_raw(ordinal)?;
    if raw.is_null() {
        return Ok(PqValue::Null);
    }
    let v = match type_name {
        "BOOL" => PqValue::Bool(row.try_get(ordinal)?),
        "INT2" | "INT4" => PqValue::Int(row.try_get::<i32, _>(ordinal)? as i64),
        "INT8" => PqValue::Int(row.try_get::<i64, _>(ordinal)?),
        "FLOAT4" => PqValue::Float(row.try_get::<f32, _>(ordinal)? as f64),
        "FLOAT8" => PqValue::Float(row.try_get::<f64, _>(ordinal)?),
        // NUMERIC 转文本，避免 0.1+0.2 这类精度问题
        "NUMERIC" | "DECIMAL" => PqValue::Text(row.try_get::<String, _>(ordinal)?),
        "TEXT" | "VARCHAR" | "BPCHAR" | "NAME" => PqValue::Text(row.try_get::<String, _>(ordinal)?),
        "UUID" => PqValue::Text(row.try_get::<String, _>(ordinal)?),
        "JSON" | "JSONB" => PqValue::Json(row.try_get::<Value, _>(ordinal)?),
        "TIMESTAMP" => {
            PqValue::Text(row.try_get::<chrono::NaiveDateTime, _>(ordinal)?.to_string())
        }
        "TIMESTAMPTZ" => PqValue::Text(
            row.try_get::<chrono::DateTime<chrono::Utc>, _>(ordinal)?
                .to_rfc3339(),
        ),
        "DATE" => PqValue::Text(row.try_get::<chrono::NaiveDate, _>(ordinal)?.to_string()),
        "TIME" => PqValue::Text(row.try_get::<chrono::NaiveTime, _>(ordinal)?.to_string()),
        "BYTEA" => PqValue::Bytes(row.try_get::<Vec<u8>, _>(ordinal)?),
        other => PqValue::Other(other.to_string()),
    };
    Ok(v)
}

/// 单行 → JSON 对象（列名作 key，灵活值作 value）。
pub fn row_to_json(row: &sqlx::postgres::PgRow) -> Result<Value> {
    let mut obj = Map::new();
    for col in row.columns() {
        let val = decode_column(row, col.ordinal(), col.type_info().name())?;
        obj.insert(col.name().to_string(), val.to_json());
    }
    Ok(Value::Object(obj))
}

/// 把 [`PqValue`] 绑定到查询参数（自动按类型选择占位符绑定，避免 SQL 注入）。
///
/// 泛型 CRUD 内部都走它，所以上层只需关心「值是什么」，不用手写 `bind`。
pub fn bind_value<'q>(mut q: PgQuery<'q>, v: &'q PqValue) -> PgQuery<'q> {
    q = match v {
        PqValue::Bool(b) => q.bind(*b),
        PqValue::Int(i) => q.bind(*i),
        PqValue::Float(f) => q.bind(*f),
        PqValue::Text(s) => q.bind(s.as_str()),
        PqValue::Json(j) => q.bind(j.clone()),
        PqValue::Null => q.bind(None::<String>),
        PqValue::Bytes(b) => q.bind(b.clone()),
        PqValue::Other(s) => q.bind(s.as_str()),
    };
    q
}

/// 执行任意 SQL（带参数），返回 JSON 数组。这是「灵活」的核心入口。
///
/// 示例：
/// ```ignore
/// let rows = fetch_json(&pool,
///     "SELECT username, balance FROM users WHERE balance > $1 ORDER BY balance DESC",
///     &[PqValue::Int(150)]).await?;
/// ```
pub async fn fetch_json(pool: &PgPool, sql: &str, params: &[PqValue]) -> Result<Vec<Value>> {
    let mut q = sqlx::query(sqlx::AssertSqlSafe(sql.to_string()));
    for v in params {
        q = bind_value(q, v);
    }
    let rows = q.fetch_all(pool).await?;
    rows.iter().map(row_to_json).collect()
}

/// 便捷插入一行。`row` 为 (列名, 值) 映射。
///
/// `on_conflict` 可传入 `Some("(username) DO UPDATE SET balance = EXCLUDED.balance")`
/// 之类子句，让重复运行更安全；传 `None` 则为普通 INSERT。
pub async fn insert(
    pool: &PgPool,
    table: &str,
    row: &[(&str, PqValue)],
    on_conflict: Option<&str>,
) -> Result<u64> {
    if row.is_empty() {
        return Err(anyhow!("insert 至少需要一列"));
    }
    let cols: Vec<&str> = row.iter().map(|(c, _)| *c).collect();
    let placeholders: Vec<String> = (1..=row.len()).map(|i| format!("${}", i)).collect();
    let mut sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table,
        cols.join(", "),
        placeholders.join(", ")
    );
    if let Some(oc) = on_conflict {
        sql.push_str(&format!(" ON CONFLICT {}", oc));
    }

    let mut q = sqlx::query(sqlx::AssertSqlSafe(sql));
    for (_, v) in row {
        q = bind_value(q, v);
    }
    Ok(q.execute(pool).await?.rows_affected())
}

/// 查询整张表，返回 JSON 数组。
pub async fn find_all(pool: &PgPool, table: &str) -> Result<Vec<Value>> {
    let sql = format!("SELECT * FROM {}", table);
    fetch_json(pool, &sql, &[]).await
}

/// 按条件查询（所有条件用 `AND` 连接，等值匹配）。
///
/// `conds` 为 (列名, 值) 映射，例如 `&[("username", PqValue::Text("alice".into()))]`。
pub async fn find_where(
    pool: &PgPool,
    table: &str,
    conds: &[(&str, PqValue)],
) -> Result<Vec<Value>> {
    let mut sql = format!("SELECT * FROM {}", table);
    let mut params: Vec<PqValue> = Vec::new();
    if !conds.is_empty() {
        let clauses: Vec<String> = conds
            .iter()
            .enumerate()
            .map(|(i, (c, _))| format!("{} = ${}", c, i + 1))
            .collect();
        sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
        params = conds.iter().map(|(_, v)| v.clone()).collect();
    }
    fetch_json(pool, &sql, &params).await
}

/// 按条件更新。`set` 为要改的字段，`conds` 为 WHERE 条件（全部 `AND` 等值）。
pub async fn update_where(
    pool: &PgPool,
    table: &str,
    set: &[(&str, PqValue)],
    conds: &[(&str, PqValue)],
) -> Result<u64> {
    if set.is_empty() {
        return Err(anyhow!("update 至少需要一个 SET 字段"));
    }
    let set_clause: Vec<String> = set
        .iter()
        .enumerate()
        .map(|(i, (c, _))| format!("{} = ${}", c, i + 1))
        .collect();
    let cond_clause: Vec<String> = conds
        .iter()
        .enumerate()
        .map(|(i, (c, _))| format!("{} = ${}", c, i + 1 + set.len()))
        .collect();
    let sql = format!(
        "UPDATE {} SET {} WHERE {}",
        table,
        set_clause.join(", "),
        if cond_clause.is_empty() {
            "TRUE".to_string()
        } else {
            cond_clause.join(" AND ")
        }
    );

    let mut q = sqlx::query(sqlx::AssertSqlSafe(sql));
    for (_, v) in set {
        q = bind_value(q, v);
    }
    for (_, v) in conds {
        q = bind_value(q, v);
    }
    Ok(q.execute(pool).await?.rows_affected())
}

/// 按条件删除，返回受影响行数。
pub async fn delete_where(
    pool: &PgPool,
    table: &str,
    conds: &[(&str, PqValue)],
) -> Result<u64> {
    let mut sql = format!("DELETE FROM {}", table);
    let mut params: Vec<PqValue> = Vec::new();
    if !conds.is_empty() {
        let clauses: Vec<String> = conds
            .iter()
            .enumerate()
            .map(|(i, (c, _))| format!("{} = ${}", c, i + 1))
            .collect();
        sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
        params = conds.iter().map(|(_, v)| v.clone()).collect();
    }
    let mut q = sqlx::query(sqlx::AssertSqlSafe(sql));
    for v in &params {
        q = bind_value(q, v);
    }
    Ok(q.execute(pool).await?.rows_affected())
}
