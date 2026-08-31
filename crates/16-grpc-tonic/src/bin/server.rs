//! gRPC 服务端：实现 Greeter 服务。
//!
//! 运行：`cargo run --bin grpc-server`，然后用 `grpc-client` 调用。

use grpc_tonic::hello::{
    greeter_server::{Greeter, GreeterServer},
    HelloReply, HelloRequest,
};
use tonic::{Request, Response, Status, transport::Server};

#[derive(Default)]
struct GreeterService;

#[tonic::async_trait]
impl Greeter for GreeterService {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let name = request.into_inner().name;

        // 参数校验失败时返回标准 gRPC 状态码，客户端可以精确匹配
        if name.trim().is_empty() {
            return Err(Status::invalid_argument("name 不能为空"));
        }

        println!("收到问候请求: name='{}'", name);
        Ok(Response::new(HelloReply {
            message: format!("Hello, {}!", name),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    println!("gRPC 服务正在监听: http://{}", addr);

    Server::builder()
        .add_service(GreeterServer::new(GreeterService::default()))
        .serve(addr)
        .await?;

    Ok(())
}
