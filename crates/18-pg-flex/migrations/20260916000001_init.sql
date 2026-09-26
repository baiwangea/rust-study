-- 演示表：用户账户（balance 用于灵活查询 / 事务演示）
-- 重复执行安全：CREATE TABLE IF NOT EXISTS + 迁移机制保证幂等
CREATE TABLE IF NOT EXISTS users (
    id       SERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    balance  BIGINT NOT NULL DEFAULT 0,
    active   BOOLEAN NOT NULL DEFAULT TRUE
);
