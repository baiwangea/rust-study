use redis::{Commands, RedisResult};

fn main() -> RedisResult<()> {
    // 连接到 Redis
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;

    println!("连接 Redis 成功");

    // === String 操作 ===
    println!("\n--- String (SET/GET) ---");
    let _: () = con.set("my_key", "hello world")?;
    let value: String = con.get("my_key")?;
    println!("GET 'my_key': {}", value);

    // === Hash 操作 ===
    println!("\n--- Hash (HSET/HGET) ---");
    let hash_key = "my_hash";
    let _: () = con.hset(hash_key, "field1", "value1")?;
    let _: () = con.hset(hash_key, "field2", "value2")?;

    let field1_val: String = con.hget(hash_key, "field1")?;
    let field2_val: String = con.hget(hash_key, "field2")?;
    println!("HGET '{}', 'field1': {}", hash_key, field1_val);
    println!("HGET '{}', 'field2': {}", hash_key, field2_val);

    println!("\nRedis 操作示例执行完毕。");

    Ok(())
}
