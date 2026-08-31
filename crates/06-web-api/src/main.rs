//! 现代 Web API 实战（axum 0.8 + JWT）。
//!
//! 路由：
//! - `GET  /`         公共路由：欢迎页
//! - `GET  /healthz`  健康检查
//! - `POST /login`    登录并签发 JWT（1 小时过期）
//! - `GET  /profile`  受保护路由：需携带 `Authorization: Bearer <token>`
//!
//! 工程实践：统一错误类型、tower-http 中间件（请求日志 + CORS）、优雅关闭。
//!
//! 试用：
//! ```bash
//! curl -X POST localhost:3000/login
//! curl -H "Authorization: Bearer <token>" localhost:3000/profile
//! ```

use axum::{
    extract::{FromRequestParts, Request},
    http::{StatusCode, header, request::Parts},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// JWT 的声明 (Claims)
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // 主题 (Subject)，这里存用户 ID
    exp: usize,  // 过期时间 (Expiration Time)
}

/// 登录成功后返回的 Token
#[derive(Debug, Serialize)]
struct TokenResponse {
    token: String,
}

/// 受保护路由返回的用户信息
#[derive(Debug, Serialize)]
struct UserProfile {
    user_id: String,
}

/// JWT 密钥，在生产环境中应从环境变量/密钥管理服务读取
const JWT_SECRET: &[u8] = b"your-secret-key";

// ==================== 统一错误类型 ====================

/// 应用层错误：实现 `IntoResponse` 后可在 handler 中直接用 `?` 传播
#[derive(Debug)]
enum ApiError {
    Unauthorized(&'static str),
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "内部错误"),
        };
        // 统一的 JSON 错误响应格式
        let body = Json(serde_json::json!({ "error": message }));
        (status, body).into_response()
    }
}

// ==================== 路由与启动 ====================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(root))
        .route("/healthz", get(healthz))
        .route("/login", post(login))
        .route("/profile", get(profile))
        // 请求日志中间件：打印每个请求的方法、路径与响应耗时
        .layer(
            TraceLayer::new_for_http()
                .on_request(|req: &Request, _: &tracing::Span| {
                    println!("--> {} {}", req.method(), req.uri());
                })
                .on_response(|res: &Response, latency: std::time::Duration, _: &tracing::Span| {
                    println!("<-- {} ({:?})", res.status(), latency);
                }),
        )
        // 中间件按添加的逆序执行：请求先经过 Trace，再经过 CORS
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("服务正在监听 http://127.0.0.1:3000 （Ctrl-C 优雅关闭）");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    println!("服务已安全停止");
    Ok(())
}

/// Ctrl-C 信号：触发后 axum 会停止接受新连接并等待存量请求完成
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("安装 ctrl-c 信号处理器失败");
    println!("\n收到 Ctrl-C，正在优雅关闭...");
}

// ==================== Handlers ====================

async fn root() -> &'static str {
    "欢迎来到 Web API 服务!"
}

async fn healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

/// 登录并签发 JWT（真实项目中这里应校验用户名/密码）
async fn login() -> Result<Json<TokenResponse>, ApiError> {
    let claims = Claims {
        sub: "user123".to_owned(),
        // 设置 token 1 小时后过期
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .map_err(|_| ApiError::Internal)?;

    Ok(Json(TokenResponse { token }))
}

/// 受保护路由：`AuthUser` 提取器会先验证 JWT
async fn profile(AuthUser(user): AuthUser) -> Json<UserProfile> {
    Json(UserProfile { user_id: user.sub })
}

// ==================== 自定义 JWT 提取器 ====================

/// 从请求头解析并验证 JWT，成功则携带 Claims 进入 handler
struct AuthUser(Claims);

// axum 0.8 的 FromRequestParts 是原生 async trait，无需 `#[async_trait]`
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 从 `Authorization: Bearer <token>` 请求头中提取 token
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError::Unauthorized("缺少 Authorization header"))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(ApiError::Unauthorized("Authorization 格式应为: Bearer <token>"))?;

        // 解码并验证签名与过期时间
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET),
            &Validation::default(),
        )
        .map_err(|_| ApiError::Unauthorized("无效的 token"))?;

        Ok(AuthUser(token_data.claims))
    }
}
