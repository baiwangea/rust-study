//! gRPC 示例（tonic）：共享的生成代码模块。
//!
//! `tonic::include_proto!` 引入 build.rs 编译 .proto 生成的代码。

pub mod hello {
    tonic::include_proto!("hello");
}
