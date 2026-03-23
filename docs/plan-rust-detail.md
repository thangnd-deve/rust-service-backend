# Story Reader Platform — Detailed Implementation Plan (Rust)

> Rust + React | 38 tasks | ~6 tuần
>
> Plan dành cho người mới học Rust, mỗi task có hướng dẫn chi tiết

---

## Trước khi bắt đầu — Setup môi trường

```bash
# Cài Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Kiểm tra
rustc --version
cargo --version

# Cài tools hỗ trợ
cargo install cargo-watch    # auto-reload khi save file
cargo install sqlx-cli       # CLI cho database migrations

# IDE: VS Code + extension "rust-analyzer"
```

### Rust concepts cần biết trước (học dần trong quá trình làm)

| Concept | Khi nào gặp | Mức độ |
| --- | --- | --- |
| Ownership & Borrowing | Ngay từ đầu | Quan trọng nhất |
| Structs & Enums | Phase 1 (models) | Cơ bản |
| Traits | Phase 1-2 (FromRow, Serialize) | Cơ bản |
| Error handling (Result, ?) | Phase 1+ | Cơ bản |
| Async/Await | Phase 2+ (Axum, SQLx) | Trung bình |
| Lifetimes | Khi compiler báo lỗi | Trung bình |
| Generics | Phase 2+ (middleware) | Trung bình |
| Closures | Phase 3+ (handlers) | Cơ bản |

Tài liệu: <https://doc.rust-lang.org/book/> (The Rust Book — đọc chapter 1-10 là đủ bắt đầu)

---

## Phase 0 — Setup & Design (2-3 ngày)

### Task 0.1: Khởi tạo Cargo workspace

**Mục tiêu:** Tạo monorepo với 4 crates

```bash
mkdir story-reader && cd story-reader

# Tạo workspace root
cargo init --name sto`ry-reader

# Tạo các crates
cargo init crates/common --lib
cargo init crates/api-server
cargo init crates/audio-worker
cargo init crates/audio-stream
```

**File `Cargo.toml` ở root:**

```toml
[workspace]
members = [
    "crates/common",
    "crates/api-server",
    "crates/audio-worker",
    "crates/audio-stream",
]
resolver = "2"

# Dependencies dùng chung, khai báo 1 lần
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "chrono", "uuid"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

**File `crates/common/Cargo.toml`:**

```toml
[package]
name = "common"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio.workspace = true        # kế thừa từ workspace
serde.workspace = true
sqlx.workspace = true
# ... các deps khác
```

**Kiểm tra:**

```bash
cargo build --workspace
# → Compiling common v0.1.0
# → Compiling api-server v0.1.0
# → Compiling audio-worker v0.1.0
# → Compiling audio-stream v0.1.0
# Không có error = OK
```

**Rust concept học được:** workspace, crate, Cargo.toml, dependency management

---

### Task 0.2: Docker Compose cho infrastructure

**Mục tiêu:** PostgreSQL + Redis + MinIO chạy local

**File `docker-compose.yml`:**

```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: story
      POSTGRES_PASSWORD: story123
      POSTGRES_DB: story_reader
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U story"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5

  minio:
    image: minio/minio:latest
    command: server /data --console-address ":9001"
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: minioadmin
    ports:
      - "9000:9000"
      - "9001:9001"    # MinIO Console UI
    volumes:
      - miniodata:/data

volumes:
  pgdata:
  miniodata:
```

**Kiểm tra:**

```bash
docker compose up -d
docker compose ps          # 3 services running
# Test connections:
psql postgresql://story:story123@localhost:5432/story_reader
redis-cli ping             # PONG
curl http://localhost:9000  # MinIO response
```

---

### Task 0.3: Database schema design

**Mục tiêu:** Tạo migration files cho tất cả tables

```bash
# Tạo thư mục migrations
mkdir migrations

# Set DATABASE_URL
export DATABASE_URL="postgresql://story:story123@localhost:5432/story_reader"

# Tạo migration
sqlx migrate add create_users
sqlx migrate add create_stories
sqlx migrate add create_chapters
sqlx migrate add create_audio_files
sqlx migrate add create_bookmarks
sqlx migrate add create_reading_history
```

**Schema chi tiết:**

```sql
-- migrations/0001_create_users.sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- migrations/0002_create_stories.sql
CREATE TABLE stories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(500) NOT NULL,
    description TEXT,
    cover_image_url VARCHAR(1000),
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',  -- draft, published, archived
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_stories_author ON stories(author_id);
CREATE INDEX idx_stories_status ON stories(status);

-- migrations/0003_create_chapters.sql
CREATE TABLE chapters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    story_id UUID NOT NULL REFERENCES stories(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    content TEXT NOT NULL,
    chapter_order INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(story_id, chapter_order)
);
CREATE INDEX idx_chapters_story ON chapters(story_id);

-- migrations/0004_create_audio_files.sql
CREATE TABLE audio_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chapter_id UUID UNIQUE NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
    file_path VARCHAR(1000) NOT NULL,
    file_size BIGINT,
    duration_seconds INT,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending, processing, ready, failed
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- migrations/0005_create_bookmarks.sql
CREATE TABLE bookmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    chapter_id UUID NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, chapter_id)
);
CREATE INDEX idx_bookmarks_user ON bookmarks(user_id);

-- migrations/0006_create_reading_history.sql
CREATE TABLE reading_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    chapter_id UUID NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
    read_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_history_user ON reading_history(user_id);
CREATE INDEX idx_history_user_chapter ON reading_history(user_id, chapter_id);
```

**Kiểm tra:** Tất cả migration files đã được tạo đúng tên và nội dung SQL hợp lệ — server sẽ tự chạy khi khởi động

---

### Task 0.4: Architecture diagram

**Mục tiêu:** Vẽ luồng data giữa các service

```text
┌──────────┐     ┌─────────────┐     ┌──────────┐
│  React   │────▶│  API Server │────▶│ Postgres │
│ Frontend │     │   (Axum)    │────▶│          │
└──────────┘     │  port 8080  │     └──────────┘
     │           └──────┬──────┘
     │                  │ publish message
     │                  ▼
     │           ┌─────────────┐     ┌──────────┐
     │           │    Redis    │────▶│  Audio   │
     │           │   Streams   │     │  Worker  │
     │           └─────────────┘     └────┬─────┘
     │                                    │ TTS API + upload
     │           ┌─────────────┐     ┌────▼─────┐
     └──────────▶│Audio Stream │────▶│  MinIO   │
                 │   (Axum)    │     │ (S3-like)│
                 │  port 8081  │     └──────────┘
                 └─────────────┘
```

Lưu vào `docs/architecture.md` hoặc dùng Excalidraw

---

### Task 0.5: API contract (OpenAPI)

**Mục tiêu:** Định nghĩa tất cả endpoints

```yaml
# Tóm tắt endpoints — chi tiết viết trong openapi.yaml

# Auth
POST   /auth/register          # { email, username, password }
POST   /auth/login              # { email, password } → { token }

# Stories
GET    /stories                 # ?page=1&per_page=20&status=published
GET    /stories/:id             # story detail
POST   /stories                 # [auth] create story
PUT    /stories/:id             # [auth] update story
DELETE /stories/:id             # [auth] delete story

# Chapters
GET    /stories/:id/chapters    # list chapters of story
GET    /chapters/:id            # chapter detail + content
POST   /stories/:id/chapters    # [auth] create chapter → trigger audio queue
PUT    /chapters/:id            # [auth] update chapter
DELETE /chapters/:id            # [auth] delete chapter

# Audio
GET    /audio/:chapter_id       # stream audio file (port 8081)

# Bookmarks
GET    /bookmarks               # [auth] list user bookmarks
POST   /bookmarks               # [auth] { chapter_id }
DELETE /bookmarks/:id           # [auth] remove bookmark

# Reading History
GET    /history                 # [auth] list reading history
POST   /history                 # [auth] { chapter_id }
```

---

## Phase 1 — Common Crate + DB Layer (2-3 ngày)

### Task 1.1: Config + DB pool

**Mục tiêu:** Load config từ `.env`, tạo DB connection pool

**File `.env`:**

```env
DATABASE_URL=postgresql://story:story123@localhost:5432/story_reader
REDIS_URL=redis://localhost:6379
MINIO_ENDPOINT=http://localhost:9000
MINIO_ACCESS_KEY=minioadmin
MINIO_SECRET_KEY=minioadmin
JWT_SECRET=your-super-secret-key-change-in-production
API_PORT=8080
STREAM_PORT=8081
```

**File `crates/common/src/config.rs`:**

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub minio_endpoint: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub jwt_secret: String,
    pub api_port: u16,
    pub stream_port: u16,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenvy::dotenv().ok();
        envy::from_env()
    }
}
```

**File `crates/common/src/db.rs`:**

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}
```

**Kiểm tra:**

```bash
# Test config load
cargo run -q -p api-server
# → "Configuration loaded: AppConfig { database_url: "postgresql://...", ... }"

# Test DB connection (thêm tạm vào main.rs của api-server)
# println!("DB connected: {:?}", pool.acquire().await.is_ok());
# → "DB connected: true"
```

**Rust concept học được:** `struct`, `derive` macros, `Result<T, E>`, `async/await`

---

### Task 1.2: Models/Entities

**Mục tiêu:** Định nghĩa Rust structs map với DB tables

**File `crates/common/src/models/user.rs`:**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    #[serde(skip_serializing)]    // không trả password_hash trong JSON response
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Struct cho request body — không có id, timestamps
#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub username: String,
    pub password: String,
}
```

Tạo tương tự cho: `Story`, `Chapter`, `AudioFile`, `Bookmark`, `ReadingHistory`

**Cấu trúc thư mục `common/src/`:**

```text
common/src/
├── lib.rs           # pub mod config; pub mod db; pub mod models; pub mod error;
├── config.rs
├── db.rs
├── error.rs
└── models/
    ├── mod.rs       # pub mod user; pub mod story; ...
    ├── user.rs
    ├── story.rs
    ├── chapter.rs
    ├── audio.rs
    ├── bookmark.rs
    └── history.rs
```

**Kiểm tra:**

```bash
cargo build -p common
# → Compiling common v0.1.0 — không error

# Test serialize User → JSON (thêm tạm vào main.rs)
# let user = User { id: Uuid::new_v4(), email: "test@test.com".into(), ... };
# println!("{}", serde_json::to_string_pretty(&user).unwrap());
# → JSON output KHÔNG chứa password_hash (vì skip_serializing)
```

**Rust concept học được:** `mod` system, `pub`, `derive` traits, `#[serde(skip_serializing)]`

---

### Task 1.3: Error handling

**Mục tiêu:** Shared error type dùng chung giữa các crates

**File `crates/common/src/error.rs`:**

```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Internal(String),
    Database(sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

// Cho phép dùng `?` với sqlx::Error → tự convert sang AppError
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Database(e)
    }
}
```

**Kiểm tra:**

```bash
cargo build -p common
# → Không error — AppError compile thành công

# Test thủ công (sau khi có Axum server ở Phase 2):
# curl http://localhost:8080/nonexistent
# → {"error":"Not Found"} với status 404
```

**Rust concept học được:** `enum`, `match`, `impl Trait for Type`, `From` trait

---

### Task 1.4: Redis connection pool

**Mục tiêu:** Shared Redis pool

**File `crates/common/src/redis.rs`:**

```rust
use deadpool_redis::{Config, Pool, Runtime};

pub fn create_redis_pool(redis_url: &str) -> Pool {
    let cfg = Config::from_url(redis_url);
    cfg.create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool")
}
```

**Kiểm tra:**

```bash
cargo build --workspace
# → Compile thành công

# Test Redis connection (thêm tạm vào main.rs):
# let mut conn = redis.get().await.unwrap();
# let _: () = redis::cmd("PING").query_async(&mut conn).await.unwrap();
# println!("Redis connected!");
# → "Redis connected!"

cargo test --workspace
# → tất cả tests pass
```

---

## Phase 2 — API Server: Auth (3-4 ngày)

### Task 2.1: Axum server skeleton

**Mục tiêu:** Server chạy, health check endpoint hoạt động

**File `crates/api-server/src/main.rs`:**

```rust
use axum::{routing::get, Router, Json};
use common::{config::AppConfig, db};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber;

// Shared state cho tất cả handlers
pub struct AppState {
    pub db: sqlx::PgPool,
    pub redis: deadpool_redis::Pool,
    pub config: AppConfig,
}

#[tokio::main]
async fn main() {
    // Init logging
    tracing_subscriber::fmt::init();

    // Load config
    let config = AppConfig::from_env().expect("Failed to load config");

    // Create connections
    let pool = db::create_pool(&config.database_url).await.unwrap();
    let redis = common::redis::create_redis_pool(&config.redis_url);

    // Run migrations
    sqlx::migrate!("../../migrations").run(&pool).await.unwrap();

    let state = Arc::new(AppState { db: pool, redis, config: config.clone() });

    let app = Router::new()
        .route("/health", get(health))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.api_port);
    tracing::info!("API server listening on {}", addr);
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}
```

**Chạy thử:**

```bash
# Terminal 1: chạy server (auto-reload khi save)
cargo watch -x 'run -p api-server'

# Terminal 2: test
curl http://localhost:8080/health
# → {"status":"ok"}
```

**Rust concept học được:** `async fn`, `Arc` (shared ownership), `Router`, `#[tokio::main]`

---

### Task 2.2: Register endpoint

**Mục tiêu:** `POST /auth/register` — tạo user mới

**File `crates/api-server/src/handlers/auth.rs`:**

```rust
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::{extract::State, Json};
use common::{error::AppError, models::user::CreateUser};
use rand::rngs::OsRng;
use std::sync::Arc;
use crate::AppState;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(input.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .to_string();

    // Insert user
    let user = sqlx::query_as!(
        common::models::user::User,
        r#"INSERT INTO users (email, username, password_hash)
           VALUES ($1, $2, $3)
           RETURNING id, email, username, password_hash, created_at, updated_at"#,
        input.email,
        input.username,
        password_hash
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "username": user.username
    })))
}
```

**Kiểm tra:**

```bash
# Happy path — register thành công
curl -s -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@test.com","username":"testuser","password":"123456"}'
# → {"id":"uuid-xxx","email":"test@test.com","username":"testuser"}

# Error case — duplicate email
curl -s -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@test.com","username":"testuser2","password":"123456"}'
# → {"error":"..."} với status 400 hoặc 500
```

---

### Task 2.3: Login endpoint

**Mục tiêu:** `POST /auth/login` — verify password, trả JWT token

```rust
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use jsonwebtoken::{encode, EncodingKey, Header};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // user_id
    pub exp: usize,       // expiry timestamp
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(input): Json<LoginInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Find user
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email = $1",
        input.email
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized("Invalid credentials".into()))?;

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Argon2::default()
        .verify_password(input.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("Invalid credentials".into()))?;

    // Generate JWT
    let claims = Claims {
        sub: user.id.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "token": token })))
}
```

**Kiểm tra:**

```bash
# Login thành công
curl -s -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@test.com","password":"123456"}'
# → {"token":"eyJhbGciOiJIUzI1NiJ9..."}

# Login sai password
curl -s -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@test.com","password":"wrong"}'
# → {"error":"Invalid credentials"} với status 401
```

---

### Task 2.4: JWT auth middleware

**Mục tiêu:** Tower middleware kiểm tra JWT, inject `user_id`

```rust
use axum::{extract::Request, middleware::Next, response::Response};
use jsonwebtoken::{decode, DecodingKey, Validation};

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized("Missing token".into()))?;

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid token".into()))?
    .claims;

    let user_id: uuid::Uuid = claims.sub.parse()
        .map_err(|_| AppError::Unauthorized("Invalid token".into()))?;

    // Inject user_id vào request extensions — handlers đọc từ đây
    request.extensions_mut().insert(user_id);

    Ok(next.run(request).await)
}

// Trong router, áp dụng middleware cho protected routes:
// .route_layer(axum::middleware::from_fn_with_state(state, auth_middleware))
```

**Kiểm tra:**

```bash
# Không có token → 401
curl -s http://localhost:8080/stories -w "\n%{http_code}"
# → {"error":"Missing token"}
# → 401

# Có token hợp lệ → 200
TOKEN="eyJhbGci..."  # lấy từ login
curl -s http://localhost:8080/stories -H "Authorization: Bearer $TOKEN" -w "\n%{http_code}"
# → 200

# Token sai → 401
curl -s http://localhost:8080/stories -H "Authorization: Bearer invalid" -w "\n%{http_code}"
# → {"error":"Invalid token"}
# → 401
```

---

### Task 2.5: Integration tests cho auth

**File `crates/api-server/tests/auth_test.rs`:**

```rust
// Dùng reqwest để gọi API thật
// Hoặc dùng axum::test::TestClient

#[tokio::test]
async fn test_register_and_login() {
    // 1. Register
    // 2. Login với credentials vừa tạo → expect 200 + token
    // 3. Login sai password → expect 401
    // 4. Register duplicate email → expect 400
}
```

**Kiểm tra:**

```bash
cargo test -p api-server
# → test test_register_and_login ... ok
# → test result: ok. X passed; 0 failed
```

---

## Phase 3 — API Server: Story & Features (4-5 ngày)

### Task 3.1: CRUD Stories

**Pattern cho mỗi CRUD endpoint:**

```rust
// GET /stories?page=1&per_page=20
pub async fn list_stories(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Story>>, AppError> {
    let offset = (params.page - 1) * params.per_page;
    let stories = sqlx::query_as!(Story,
        "SELECT * FROM stories WHERE status = 'published'
         ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        params.per_page as i64,
        offset as i64
    )
    .fetch_all(&state.db)
    .await?;

    // count total
    let total = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM stories WHERE status = 'published'"
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(PaginatedResponse { data: stories, total: total.unwrap_or(0) }))
}

// Router setup
let story_routes = Router::new()
    .route("/stories", get(list_stories).post(create_story))
    .route("/stories/:id", get(get_story).put(update_story).delete(delete_story));
```

**Kiểm tra:**

```bash
# Tạo story
curl -s -X POST http://localhost:8080/stories \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"My Story","description":"A test story"}'
# → {"id":"uuid-xxx","title":"My Story",...}

# List stories
curl -s http://localhost:8080/stories?page=1&per_page=10
# → {"data":[...],"total":1}

# Get story detail
curl -s http://localhost:8080/stories/<story_id>
# → {"id":"...","title":"My Story",...}

# Update story
curl -s -X PUT http://localhost:8080/stories/<story_id> \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"Updated Title"}'
# → {"id":"...","title":"Updated Title",...}

# Delete story
curl -s -X DELETE http://localhost:8080/stories/<story_id> \
  -H "Authorization: Bearer $TOKEN" -w "\n%{http_code}"
# → 204 hoặc 200
```

### Task 3.2: CRUD Chapters

Tương tự Stories, thêm `story_id` filter + `chapter_order` sorting

**Kiểm tra:**

```bash
# Tạo chapter
curl -s -X POST http://localhost:8080/stories/<story_id>/chapters \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"Chapter 1","content":"Once upon a time...","chapter_order":1}'
# → {"id":"uuid-xxx","title":"Chapter 1","chapter_order":1,...}

# List chapters by story
curl -s http://localhost:8080/stories/<story_id>/chapters
# → [{"id":"...","title":"Chapter 1","chapter_order":1},...]
```

### Task 3.3: Bookmarks API

**Kiểm tra:**

```bash
# Tạo bookmark
curl -s -X POST http://localhost:8080/bookmarks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"chapter_id":"<chapter_id>"}'
# → {"id":"uuid-xxx","chapter_id":"...","created_at":"..."}

# List bookmarks
curl -s http://localhost:8080/bookmarks -H "Authorization: Bearer $TOKEN"
# → [{"id":"...","chapter_id":"...",...}]

# Delete bookmark
curl -s -X DELETE http://localhost:8080/bookmarks/<bookmark_id> \
  -H "Authorization: Bearer $TOKEN" -w "\n%{http_code}"
# → 204
```

### Task 3.4: Reading History API

**Kiểm tra:**

```bash
# Ghi history
curl -s -X POST http://localhost:8080/history \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"chapter_id":"<chapter_id>"}'
# → {"id":"...","chapter_id":"...","read_at":"..."}

# List history
curl -s http://localhost:8080/history -H "Authorization: Bearer $TOKEN"
# → [{"chapter_id":"...","read_at":"2026-03-20T..."},...]
```

### Task 3.5: Publish lên Redis Streams

```rust
use deadpool_redis::redis::cmd;

pub async fn publish_chapter_created(
    redis: &deadpool_redis::Pool,
    chapter_id: uuid::Uuid,
    story_id: uuid::Uuid,
    text: &str,
) -> Result<(), AppError> {
    let mut conn = redis.get().await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    cmd("XADD")
        .arg("chapter:audio")           // stream name
        .arg("*")                       // auto-generate ID
        .arg("chapter_id")
        .arg(chapter_id.to_string())
        .arg("story_id")
        .arg(story_id.to_string())
        .arg("text")
        .arg(text)
        .query_async::<()>(&mut conn)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}
```

**Kiểm tra:**

```bash
# Tạo chapter mới → message tự publish
curl -s -X POST http://localhost:8080/stories/<story_id>/chapters \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"Chapter 2","content":"Text content...","chapter_order":2}'

# Kiểm tra Redis stream có message
redis-cli XRANGE chapter:audio - +
# → 1) "1679-0"
# →    1) "chapter_id" 2) "uuid-xxx" 3) "story_id" 4) "uuid-yyy" 5) "text" 6) "Text content..."
```

### Task 3.6: Redis caching

```rust
use deadpool_redis::redis::AsyncCommands;

pub async fn get_story_cached(
    state: &AppState,
    story_id: uuid::Uuid,
) -> Result<Story, AppError> {
    let cache_key = format!("story:{}", story_id);
    let mut redis_conn = state.redis.get().await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Try cache first
    if let Ok(cached) = redis_conn.get::<_, String>(&cache_key).await {
        if let Ok(story) = serde_json::from_str(&cached) {
            return Ok(story);
        }
    }

    // Cache miss → query DB
    let story = sqlx::query_as!(Story, "SELECT * FROM stories WHERE id = $1", story_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("Story not found".into()))?;

    // Set cache với TTL 5 phút
    let _: () = redis_conn.set_ex(
        &cache_key,
        serde_json::to_string(&story).unwrap(),
        300
    ).await.map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(story)
}
```

**Kiểm tra:**

```bash
# Lần 1 — cache miss (query DB)
curl -s http://localhost:8080/stories/<story_id>
# → response bình thường

# Kiểm tra Redis đã cache
redis-cli GET story:<story_id>
# → JSON string của story

# Lần 2 — cache hit (không query DB, nhanh hơn)
curl -s http://localhost:8080/stories/<story_id>
# → cùng response, nhưng nhanh hơn (< 5ms)

# Update story → cache bị xóa
curl -s -X PUT http://localhost:8080/stories/<story_id> \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"New Title"}'
redis-cli GET story:<story_id>
# → (nil) — cache đã bị invalidate
```

---

## Phase 4 — Audio Worker (4-6 ngày)

### Task 4.1: Worker binary + graceful shutdown

```rust
// crates/audio-worker/src/main.rs
use tokio::signal;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = AppConfig::from_env().unwrap();
    let db = db::create_pool(&config.database_url).await.unwrap();
    let redis = create_redis_pool(&config.redis_url);

    tracing::info!("Audio worker started");

    tokio::select! {
        _ = consume_loop(&db, &redis, &config) => {},
        _ = signal::ctrl_c() => {
            tracing::info!("Shutting down gracefully...");
        }
    }
}
```

**Kiểm tra:**

```bash
cargo run -q -p audio-worker
# → "Audio worker started"
# → worker chạy liên tục, chờ message

# Ctrl+C
# → "Shutting down gracefully..."
# → process thoát sạch, không panic
```

### Task 4.2: Redis Streams consumer (XREADGROUP)

```rust
async fn consume_loop(db: &PgPool, redis: &Pool, config: &AppConfig) {
    let mut conn = redis.get().await.unwrap();

    // Tạo consumer group (ignore error nếu đã tồn tại)
    let _: Result<(), _> = cmd("XGROUP")
        .arg("CREATE").arg("chapter:audio").arg("audio-workers").arg("0")
        .arg("MKSTREAM")
        .query_async(&mut conn).await;

    loop {
        // Đọc message
        let result: Vec<StreamReadReply> = cmd("XREADGROUP")
            .arg("GROUP").arg("audio-workers").arg("worker-1")
            .arg("COUNT").arg(1)
            .arg("BLOCK").arg(5000)    // block 5 giây
            .arg("STREAMS").arg("chapter:audio").arg(">")
            .query_async(&mut conn).await.unwrap_or_default();

        for message in result {
            // Parse chapter_id, text
            // Process: TTS → upload → update DB
            // XACK khi xong
        }
    }
}
```

**Kiểm tra:**

```bash
# Terminal 1: chạy worker
cargo run -q -p audio-worker

# Terminal 2: push message thủ công vào Redis
redis-cli XADD chapter:audio '*' chapter_id "test-id" story_id "story-id" text "Hello world"

# Terminal 1 sẽ log:
# → "Received message: chapter_id=test-id"
```

### Task 4.3: TTS API integration

```rust
async fn text_to_speech(text: &str, api_key: &str) -> Result<Vec<u8>, AppError> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/audio/speech")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": "tts-1",
            "input": text,
            "voice": "alloy",
            "response_format": "mp3"
        }))
        .send()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let bytes = response.bytes().await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(bytes.to_vec())
}
```

**Kiểm tra:**

```bash
# Test trực tiếp bằng unit test hoặc thêm tạm vào worker:
# let bytes = text_to_speech("Hello world", &api_key).await.unwrap();
# println!("Audio size: {} bytes", bytes.len());
# → "Audio size: 24576 bytes" (khoảng 20-50KB cho text ngắn)
```

### Task 4.4: Upload to MinIO

```rust
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;

async fn upload_audio(
    s3: &S3Client,
    chapter_id: &str,
    audio_bytes: Vec<u8>,
) -> Result<String, AppError> {
    let key = format!("audio/{}.mp3", chapter_id);

    s3.put_object()
        .bucket("story-audio")
        .key(&key)
        .body(ByteStream::from(audio_bytes))
        .content_type("audio/mpeg")
        .send()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(key)
}
```

**Kiểm tra:**

```bash
# Kiểm tra MinIO Console (browser)
# http://localhost:9001 → login minioadmin/minioadmin
# → Bucket "story-audio" → file "audio/<chapter_id>.mp3" tồn tại

# Hoặc dùng mc CLI
mc ls local/story-audio/audio/
# → [2026-03-20] 24.5KiB audio/<chapter_id>.mp3
```

### Task 4.5: Update DB sau upload

**Kiểm tra:**

```bash
# Query DB sau khi worker xử lý xong
psql $DATABASE_URL -c "SELECT id, chapter_id, status, file_path FROM audio_files;"
# → id | chapter_id | status | file_path
# → ...|    ...     | ready  | audio/<chapter_id>.mp3
```

### Task 4.6: Retry logic với `backon`

```rust
use backon::{ExponentialBuilder, Retryable};

let audio_bytes = (|| async { text_to_speech(&text, &api_key).await })
    .retry(ExponentialBuilder::default().with_max_times(3))
    .await?;
```

**Kiểm tra:**

```bash
# Tạm tắt network/API key sai → worker sẽ retry
cargo run -q -p audio-worker
# → "TTS API failed, retrying (1/3)..."
# → "TTS API failed, retrying (2/3)..."
# → "TTS API failed, retrying (3/3)..."
# → "Failed after 3 retries: ..."

# Bật lại network → worker xử lý bình thường
```

---

## Phase 5 — Audio Streaming Service (4-5 ngày)

### Task 5.1: Axum server cho streaming (port 8081)

**Kiểm tra:**

```bash
cargo run -q -p audio-stream
# → "Audio stream server listening on 0.0.0.0:8081"

curl -s http://localhost:8081/health
# → {"status":"ok"}
```

### Task 5.2: Full file stream

```rust
use axum::body::Body;
use tokio_util::io::ReaderStream;

pub async fn stream_audio(
    Path(chapter_id): Path<Uuid>,
    State(state): State<Arc<StreamState>>,
) -> Result<Response<Body>, AppError> {
    let key = format!("audio/{}.mp3", chapter_id);

    let object = state.s3.get_object()
        .bucket("story-audio")
        .key(&key)
        .send()
        .await
        .map_err(|e| AppError::NotFound(e.to_string()))?;

    let stream = ReaderStream::new(object.body.into_async_read());

    Ok(Response::builder()
        .header("Content-Type", "audio/mpeg")
        .body(Body::from_stream(stream))
        .unwrap())
}
```

**Kiểm tra:**

```bash
# Stream audio file
curl -s -o test.mp3 http://localhost:8081/audio/<chapter_id>
# → file test.mp3 được tạo, play được bằng audio player

# Kiểm tra Content-Type
curl -sI http://localhost:8081/audio/<chapter_id>
# → Content-Type: audio/mpeg
```

### Task 5.3: HTTP Range Requests (206 Partial Content)

**Kiểm tra:**

```bash
# Request với Range header
curl -sI -H "Range: bytes=0-1023" http://localhost:8081/audio/<chapter_id>
# → HTTP/1.1 206 Partial Content
# → Content-Range: bytes 0-1023/...
# → Content-Length: 1024
```

### Task 5.4: Memory-efficient streaming

**Kiểm tra:**

```bash
# Upload 1 file audio lớn (~50MB) vào MinIO
# Chạy streaming server, monitor memory
cargo run -q -p audio-stream &
curl -s -o /dev/null http://localhost:8081/audio/<large_chapter_id>

# Kiểm tra memory usage — phải ổn định, không spike lên 50MB
ps aux | grep audio-stream
# → RSS ~20-30MB (không tăng theo file size)
```

### Task 5.5: Concurrent streaming test

**Kiểm tra:**

```bash
# Dùng wrk hoặc k6
wrk -t4 -c100 -d10s http://localhost:8081/audio/<chapter_id>
# → Requests/sec > 500
# → Latency avg < 200ms
# → 0 errors
```

### Task 5.6: CORS middleware

**Kiểm tra:**

```bash
# Preflight request
curl -sI -X OPTIONS http://localhost:8081/audio/<chapter_id> \
  -H "Origin: http://localhost:5173" \
  -H "Access-Control-Request-Method: GET"
# → Access-Control-Allow-Origin: http://localhost:5173
# → Access-Control-Allow-Methods: GET

# Từ React dev server (localhost:5173) — audio player không bị CORS error
```

---

## Phase 6 — React Frontend (3-5 ngày)

### Task 6.1: Setup Vite + React + Ant Design

```bash
cd frontend
npm create vite@latest . -- --template react-ts
npm install antd @tanstack/react-query axios react-router-dom
```

**Kiểm tra:**

```bash
cd frontend && npm run dev
# → Local: http://localhost:5173/
# → Browser mở ra, thấy layout Ant Design cơ bản
```

### Task 6.2: Story List page

**Kiểm tra:**

```bash
# Browser: http://localhost:5173/
# → Danh sách stories hiển thị (lấy từ API)
# → Pagination hoạt động (next/prev page)
# → Click vào story → chuyển sang trang detail
```

### Task 6.3: Chapter Reader page

**Kiểm tra:**

```bash
# Browser: http://localhost:5173/stories/<id>/chapters/<id>
# → Nội dung chapter hiển thị đầy đủ
# → Nút Next/Prev chapter hoạt động
# → Scroll hết trang → tự động mark as read
```

### Task 6.4: Audio Player component

**Kiểm tra:**

```bash
# Browser: mở chapter có audio
# → Audio player hiển thị ở dưới
# → Play/Pause hoạt động
# → Seek bar kéo được, nhảy đúng vị trí
# → Progress bar chạy theo thời gian
# → Không bị CORS error trong Console
```

### Task 6.5: Bookmarks + History

**Kiểm tra:**

```bash
# Browser: mở chapter → click Bookmark
# → Icon bookmark đổi trạng thái (đã bookmark)
# → Vào trang /bookmarks → thấy chapter vừa bookmark
# → Vào trang /history → thấy chapters đã đọc, sort theo thời gian
```

### Task 6.6: Auth pages + protected routes

**Kiểm tra:**

```bash
# Browser: http://localhost:5173/login
# → Form login hiển thị
# → Login thành công → redirect về trang chủ
# → Register thành công → redirect về login

# Chưa login → truy cập /bookmarks → redirect về /login
# JWT hết hạn → tự redirect về /login
```

---

## Phase 7 — Polish & Deploy (2-3 ngày)

### Task 7.1: Dockerfile (multi-stage build)

```dockerfile
# Build stage
FROM rust:1.82 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p api-server

# Runtime stage — nhẹ, chỉ ~20MB
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/api-server /usr/local/bin/
CMD ["api-server"]
```

**Kiểm tra:**

```bash
docker build -t api-server -f docker/Dockerfile.api-server .
docker images api-server
# → REPOSITORY    TAG     SIZE
# → api-server    latest  ~20MB

docker run --rm api-server --help
# → chạy được, không missing library
```

### Task 7.2: Docker Compose full stack

**Kiểm tra:**

```bash
docker compose up --build -d
docker compose ps
# → 6 services running (postgres, redis, minio, api-server, audio-worker, audio-stream)
# → Không có port conflict

curl http://localhost:8080/health
# → {"status":"ok"}

curl http://localhost:8081/health
# → {"status":"ok"}
```

### Task 7.3: README

**Kiểm tra:**

```bash
# Clone repo mới vào thư mục khác, chỉ đọc README và làm theo:
# 1. docker compose up -d → infra chạy
# 2. cargo run -p api-server → server chạy
# Nếu trong 10 phút setup xong = README đủ tốt
```

### Task 7.4: Seed data

**Kiểm tra:**

```bash
# Chạy seed script
cargo run -q -p api-server -- --seed
# hoặc: sqlx ... < seeds/data.sql

# Verify
curl -s http://localhost:8080/stories | jq '.total'
# → 5

curl -s http://localhost:8080/stories | jq '.data[0].title'
# → "Story title..."
```

---

## Checklist tổng hợp

| Phase | Tasks | Thời gian | Status |
| --- | --- | --- | --- |
| Phase 0 — Setup | 5 tasks | 2-3 ngày | - |
| Phase 1 — Common | 4 tasks | 2-3 ngày | - |
| Phase 2 — Auth | 5 tasks | 3-4 ngày | - |
| Phase 3 — Story API | 6 tasks | 4-5 ngày | - |
| Phase 4 — Audio Worker | 6 tasks | 4-6 ngày | - |
| Phase 5 — Streaming | 6 tasks | 4-5 ngày | - |
| Phase 6 — Frontend | 6 tasks | 3-5 ngày | - |
| Phase 7 — Deploy | 4 tasks | 2-3 ngày | - |
| **Tổng** | **42 tasks** | **~6 tuần** | - |

---

## Tips khi học Rust qua project này

1. **Đừng cố hiểu hết Rust trước khi code** — code đến đâu học đến đó
2. **Compiler error là bạn** — đọc kỹ error message, Rust compiler giải thích rất rõ
3. **Clone khi bí** — gặp borrow checker error? thêm `.clone()` trước, optimize sau
4. **Dùng `cargo watch`** — `cargo watch -x 'run -p api-server'` để auto-reload
5. **Đọc docs trên docs.rs** — search crate name, đọc examples
6. **Hỏi AI** — paste compiler error, AI giải thích rất tốt
