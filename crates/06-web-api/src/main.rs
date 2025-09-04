use axum::{
    async_trait,
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

// JWT 的声明 (Claims)
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // 主题 (Subject)，这里我们存用户ID
    exp: usize,  // 过期时间 (Expiration Time)
}

// 登录成功后返回的 Token
#[derive(Debug, Serialize)]
struct TokenResponse {
    token: String,
}

// 受保护路由返回的用户信息
#[derive(Debug, Serialize, Deserialize)]
struct UserProfile {
    user_id: String,
}

// JWT 密钥，在生产环境中应该更复杂且保密
const JWT_SECRET: &[u8] = b"your-secret-key";

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/login", post(login))
        .route("/profile", get(profile));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("服务正在监听 http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// 公共路由：根路径
async fn root() -> &'static str {
    "欢迎来到 Web API 服务!"
}

// 公共路由：登录并获取 JWT
async fn login() -> Json<TokenResponse> {
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
    .unwrap();

    Json(TokenResponse { token })
}

// 受保护路由：获取用户个人信息
// `AuthUser` 是我们自定义的提取器，它会验证 JWT 并返回用户信息
async fn profile(AuthUser(user): AuthUser) -> Json<UserProfile> {
    Json(UserProfile { user_id: user.sub })
}

// --- 自定义 JWT 提取器 ---
struct AuthUser(Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 从请求头中提取 `Authorization: Bearer <token>`
        let TypedHeader(Authorization(bearer)) = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "缺少 Authorization header"))?;

        // 解码和验证 token
        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(JWT_SECRET),
            &Validation::default(),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "无效的 token"))?;

        Ok(AuthUser(token_data.claims))
    }
}
