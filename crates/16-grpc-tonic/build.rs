//! 构建脚本：编译 .proto 文件生成 Rust 代码（需要本机安装 protoc）
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/hello.proto")?;
    Ok(())
}
