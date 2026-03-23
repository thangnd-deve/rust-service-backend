# Story Reader Platform — Build Plan (Full Rust)

> Rust + React | 38 tasks | ~6 tuần

## Tech Stack

- **Backend**: Axum + SQLx + Tower
- **Audio Worker**: Tokio + Reqwest
- **Audio Streaming**: Axum + async byte stream
- **DB**: PostgreSQL (SQLx — compile-time checked queries)
- **Cache**: Redis (`deadpool-redis`)
- **Queue**: Redis Streams
- **Storage**: MinIO (`aws-sdk-s3`)
- **Auth**: `jsonwebtoken` + `argon2`
- **Frontend**: Vite + React + TypeScript + Ant Design + TanStack Query

## Cargo Workspace Layout

```text
Cargo.toml (workspace)
├── crates/
│   ├── common/          # shared models, errors, db pool, config
│   ├── api-server/      # user + story + bookmark API (Axum)
│   ├── audio-worker/    # queue consumer + TTS + upload
│   └── audio-stream/    # audio streaming service (Axum)
├── frontend/            # React app
├── migrations/          # SQLx migrations
└── docker/
```

---

## Phase 0 — Setup & Design `Infra` *(2–3 ngày)*

- [ ] Khởi tạo Cargo workspace với 4 crates (common, api-server, audio-worker, audio-stream)
  - ✓ Output: `cargo build --workspace` thành công, mỗi crate có `Cargo.toml` riêng
- [ ] Viết `docker-compose.yml` với PostgreSQL, Redis, MinIO
  - ✓ Output: `docker-compose.yml` chạy được, kết nối health check OK
- [ ] Thiết kế database schema (users, stories, chapters, audio_files, reading_history, bookmarks)
  - ✓ Output: SQLx migration files trong `migrations/`, có đủ FK và index cơ bản
- [ ] Vẽ system architecture diagram (draw.io hoặc Excalidraw)
  - ✓ Output: diagram lưu vào `docs/`, thể hiện rõ luồng data giữa các service
- [ ] Định nghĩa API contract (OpenAPI/Swagger)
  - ✓ Output: `openapi.yaml` với đủ endpoints cho stories, chapters, users, auth

---

## Phase 1 — Common Crate + DB Layer `Rust` *(2–3 ngày)*

- [ ] Setup `common` crate: config (dotenv), DB pool (SQLx + PgPool), error types
  - ✓ Output: shared DB pool khởi tạo thành công, config load từ `.env`
- [ ] Định nghĩa models/entities: User, Story, Chapter, AudioFile, Bookmark, ReadingHistory
  - ✓ Output: structs với `sqlx::FromRow`, `Serialize/Deserialize`, shared giữa các crates
- [ ] Viết SQLx migrations và chạy `sqlx migrate run`
  - ✓ Output: tất cả tables được tạo đúng, `sqlx prepare` generate offline query data
- [ ] Setup shared Redis connection pool (`deadpool-redis`)
  - ✓ Output: Redis pool connect thành công, có helper functions cho get/set/del

---

## Phase 2 — API Server: Auth `Rust` *(3–4 ngày)*

- [ ] Khởi tạo Axum server trong `api-server` crate, setup router + shared state
  - ✓ Output: `GET /health` trả về 200, server chạy port 8080
- [ ] Implement Register endpoint (`POST /auth/register`)
  - ✓ Output: user lưu vào DB, password hash bằng `argon2` (an toàn hơn bcrypt)
- [ ] Implement Login endpoint (`POST /auth/login`)
  - ✓ Output: trả về JWT token hợp lệ (dùng `jsonwebtoken` crate), có expiry
- [ ] Implement JWT auth middleware (Tower layer)
  - ✓ Output: middleware reject request không có token, inject `user_id` vào request extensions
- [ ] Viết integration tests cho auth
  - ✓ Output: test Register, Login happy path và error cases, dùng `axum::test` hoặc `reqwest`

---

## Phase 3 — API Server: Story & Features `Rust` *(4–5 ngày)*

- [ ] CRUD Stories (`GET` list, `GET` detail, `POST`, `PUT`, `DELETE`)
  - ✓ Output: 5 endpoints hoạt động, pagination với `LIMIT/OFFSET` hoặc cursor-based
- [ ] CRUD Chapters (`GET` list by story, `GET` detail, `POST`, `PUT`, `DELETE`)
  - ✓ Output: chapters liên kết đúng với `story_id`, sort theo `order` field
- [ ] Implement Bookmark API (`POST /bookmarks`, `DELETE /bookmarks/:id`, `GET /bookmarks`)
  - ✓ Output: user bookmark chapter, trả về danh sách bookmarks của user
- [ ] Implement Reading History (`POST /history`, `GET /history`)
  - ✓ Output: ghi lại `chapter_id` + timestamp, query được lịch sử theo user
- [ ] Publish message lên Redis Streams khi chapter mới được tạo
  - ✓ Output: message JSON `{chapter_id, story_id, text}` xuất hiện trong stream
- [ ] Implement Redis caching cho story/chapter detail
  - ✓ Output: cache hit < 5ms, cache invalidation khi update/delete

---

## Phase 4 — Audio Worker `Rust` *(4–6 ngày)*

- [ ] Khởi tạo `audio-worker` binary, setup tokio runtime + graceful shutdown
  - ✓ Output: worker start, connect Redis + DB + MinIO thành công
- [ ] Implement Redis Streams consumer (XREADGROUP)
  - ✓ Output: worker nhận message JSON, parse `chapter_id` thành công
- [ ] Integrate TTS API (Google TTS hoặc OpenAI TTS) qua `reqwest`
  - ✓ Output: gọi API với chapter text, nhận về audio bytes (mp3)
- [ ] Upload audio file lên MinIO (dùng `aws-sdk-s3`)
  - ✓ Output: file lưu với key `audio/{chapter_id}.mp3`, presigned URL hoạt động
- [ ] Cập nhật `audio_files` table sau khi upload thành công
  - ✓ Output: record trong DB có `file_path`, `duration`, `status = ready`
- [ ] Xử lý retry logic với exponential backoff (dùng `tokio-retry` hoặc `backon`)
  - ✓ Output: tối đa 3 lần retry, log rõ error qua `tracing`

---

## Phase 5 — Audio Streaming Service `Rust` *(4–5 ngày)*

- [ ] Khởi tạo Axum server trong `audio-stream` crate
  - ✓ Output: `GET /health` trả về 200 OK, server chạy port 8081
- [ ] Implement `GET /audio/:chapter_id` — full file stream
  - ✓ Output: client nhận được audio file hoàn chỉnh, Content-Type đúng
- [ ] Implement HTTP Range Requests (partial content — `206 Partial Content`)
  - ✓ Output: request với `Range` header nhận đúng byte range
- [ ] Stream audio từ MinIO qua async byte stream (không load toàn bộ vào RAM)
  - ✓ Output: dùng `tokio::io` + `StreamBody`, memory usage ổn định dù file 50MB
- [ ] Test concurrent streaming (k6 hoặc wrk)
  - ✓ Output: 100 concurrent requests không drop, latency < 200ms cho first byte
- [ ] CORS middleware (Tower)
  - ✓ Output: browser không bị CORS error khi play audio từ React

---

## Phase 6 — React Frontend `React` *(3–5 ngày)*

- [ ] Khởi tạo Vite + React + TypeScript + Ant Design + TanStack Query
  - ✓ Output: dev server chạy, có layout cơ bản với Ant Design components
- [ ] Trang Story List — browse và search stories
  - ✓ Output: danh sách stories với pagination, click vào xem detail
- [ ] Trang Chapter Reader — đọc nội dung chapter
  - ✓ Output: hiển thị text chapter, nút next/prev chapter, mark as read khi scroll hết
- [ ] Audio Player component — play/pause, seek, progress bar
  - ✓ Output: dùng HTML5 Audio API, Range Request hoạt động khi seek
- [ ] Bookmark button + Reading History page
  - ✓ Output: user bookmark được, xem lại danh sách đã đọc
- [ ] Auth pages (Login, Register) + protected routes
  - ✓ Output: JWT lưu vào httpOnly cookie (an toàn hơn localStorage), redirect nếu chưa login

---

## Phase 7 — Polish & Deploy `Infra` *(2–3 ngày)*

- [ ] Viết Dockerfile cho từng service (3 Rust binaries, 1 React)
  - ✓ Output: multi-stage build, image size < 20MB mỗi Rust binary
- [ ] Cập nhật `docker-compose.yml` để chạy full stack
  - ✓ Output: `docker compose up --build` khởi động toàn bộ, không conflict port
- [ ] Viết README với architecture diagram, setup guide, API docs
  - ✓ Output: `README.md` đủ để người khác clone về và chạy trong 10 phút
- [ ] Seed data demo (stories + chapters) để showcase
  - ✓ Output: script seed tạo 5 stories, mỗi story 3 chapters, 1 audio file mẫu

---

## Key Rust Crates

| Mục đích | Crate |
|---|---|
| Web framework | `axum` |
| Async runtime | `tokio` |
| DB queries | `sqlx` (compile-time checked) |
| Redis | `deadpool-redis` |
| S3/MinIO | `aws-sdk-s3` |
| JWT | `jsonwebtoken` |
| Password hash | `argon2` |
| HTTP client | `reqwest` |
| Serialization | `serde`, `serde_json` |
| Logging/Tracing | `tracing`, `tracing-subscriber` |
| Retry | `backon` |
| Config | `dotenvy`, `config` |
| Testing | `axum-test` hoặc `reqwest` + `testcontainers` |

---

*38 tasks tổng · ~6 tuần với AI coding assistant*
*Lợi thế: unified toolchain, shared code via workspace, performance đồng nhất*
