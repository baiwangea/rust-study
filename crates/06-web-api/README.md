# 现代化 Web API 实例 (web-api)

本项目使用 `axum` 框架构建一个简单的 Web API 服务，并演示如何使用 `jsonwebtoken` 来生成和验证 JWT (JSON Web Tokens) 以保护路由。

## API 接口

- `GET /`: 一个公共的欢迎接口。
- `POST /login`: 一个公共接口，用于模拟用户登录并获取一个 JWT。
- `GET /profile`: 一个受保护的接口，需要提供有效的 JWT 才能访问。

## 如何运行

```bash
cargo run
```

服务将启动在 `127.0.0.1:3000`。

### 如何测试

1.  **登录并获取 Token**:
    ```bash
    curl -X POST http://127.0.0.1:3000/login
    ```
    你会得到一个类似 `{"token":"..."}` 的响应。

2.  **访问受保护的接口** (将 `YOUR_TOKEN` 替换为上一步获取的 token):
    ```bash
    curl -H "Authorization: Bearer YOUR_TOKEN" http://127.0.0.1:3000/profile
    ```
