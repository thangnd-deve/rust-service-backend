# Story Reader Platform — Implementation Plan (Rust)

> Rust + React | 42 tasks | ~6 tuần
>
> Mỗi task chỉ nêu yêu cầu và output mong đợi — bạn tự implement

---

## Trước khi bắt đầu — Setup môi trường

Cài: `rustup`, `cargo-watch`, `sqlx-cli`, VS Code + extension `rust-analyzer`

Rust concepts cần biết dần trong quá trình làm:

| Concept | Khi nào gặp |
| --- | --- |
| Ownership & Borrowing | Ngay từ đầu |
| Structs & Enums | Phase 1 (models) |
| Traits | Phase 1-2 |
| Error handling (Result, ?) | Phase 1+ |
| Async/Await | Phase 2+ |
| Lifetimes | Khi compiler báo lỗi |
| Generics | Phase 2+ |
| Closures | Phase 3+ |

Tài liệu: <https://doc.rust-lang.org/book/> (chapter 1–10 là đủ bắt đầu)

---

## Phase 0 — Setup & Design (2-3 ngày)

### Task 0.1: Khởi tạo Cargo workspace

**Yêu cầu:**
- Monorepo với 4 crates: `common` (lib), `api-server`, `audio-worker`, `audio-stream`
- Shared dependencies khai báo ở workspace level (`[workspace.dependencies]`)

**Output mong đợi:**
- `cargo build --workspace` thành công, không có error
- 4 thư mục dưới `crates/` có đúng cấu trúc Cargo crate

**Rust concept học được:** workspace, crate, Cargo.toml, dependency management

---

### Task 0.2: Docker Compose cho infrastructure

**Yêu cầu:**
- 3 services: PostgreSQL 16, Redis 7, MinIO
- Có healthcheck cho postgres và redis
- Volumes persistent cho postgres và minio data

**Output mong đợi:**
- `docker compose up -d` → 3 services status `healthy`
- Connect được tới postgres, redis-cli `PING` trả `PONG`, MinIO Console mở được tại port 9001

---

### Task 0.3: Database schema design

**Yêu cầu:**
- 6 migration files: `users`, `stories`, `chapters`, `audio_files`, `bookmarks`, `reading_history`
- Đúng quan hệ FK, indexes cho các cột query thường xuyên
- Dùng `UUID` cho primary key, `TIMESTAMPTZ` cho timestamps

**Tables tối thiểu:**
- `users`: id, email (unique), username (unique), password_hash, timestamps
- `stories`: id, title, description, cover_image_url, author_id (FK users), status (draft/published/archived), timestamps
- `chapters`: id, story_id (FK stories), title, content, chapter_order, timestamps — unique(story_id, chapter_order)
- `audio_files`: id, chapter_id (FK unique), file_path, file_size, duration_seconds, status (pending/processing/ready/failed), error_message, timestamps
- `bookmarks`: id, user_id (FK), chapter_id (FK) — unique(user_id, chapter_id)
- `reading_history`: id, user_id (FK), chapter_id (FK), read_at

**Output mong đợi:**
- `sqlx migrate run` thành công, không có error
- Tất cả tables tồn tại trong database với đúng columns

---

### Task 0.4: Architecture diagram

**Yêu cầu:**
- Vẽ luồng data giữa các thành phần: React → API Server → Postgres/Redis → Audio Worker → MinIO → Audio Stream → React

**Output mong đợi:**
- File `docs/architecture.md` hoặc diagram trong Excalidraw/draw.io
- Thể hiện rõ port của từng service, hướng data flow

---

### Task 0.5: API contract

**Yêu cầu:**
- Định nghĩa tất cả endpoints với method, path, auth requirement, request/response shape

**Endpoints cần có:**
- Auth: `POST /auth/register`, `POST /auth/login`
- Stories: CRUD đầy đủ, có phân trang, filter theo status
- Chapters: CRUD, nested dưới story
- Audio: `GET /audio/:chapter_id` (stream, port 8081)
- Bookmarks: list, create, delete
- Reading history: list, create

**Output mong đợi:**
- File `docs/openapi.yaml` hoặc `docs/api-contract.md` đủ để frontend dev implement client mà không cần hỏi

---

## Phase 1 — Common Crate + DB Layer (2-3 ngày)

### Task 1.1: Config + DB pool

**Yêu cầu:**
- `AppConfig` struct load từ environment variables (dùng `dotenvy` + `envy`)
- Function tạo `PgPool` với connection limit hợp lý
- File `.env` với đủ các biến: `DATABASE_URL`, `REDIS_URL`, `MINIO_*`, `JWT_SECRET`, `API_PORT`, `STREAM_PORT`

**Output mong đợi:**
- `cargo run -p api-server` log ra config đã load (không leak secret)
- DB connection thành công (không panic)

**Rust concept học được:** `struct`, `derive` macros, `Result<T, E>`, `async/await`

---

### Task 1.2: Models/Entities

**Yêu cầu:**
- Rust structs mapping với 6 DB tables, derive `FromRow`, `Serialize`, `Deserialize`
- `password_hash` không serialize ra JSON response
- Separate "request" structs (không có id, timestamps) cho create operations
- Module structure rõ ràng: `common/src/models/`

**Output mong đợi:**
- `cargo build -p common` không error
- Serialize `User` ra JSON → không có field `password_hash` trong output

**Rust concept học được:** `mod` system, `pub`, `derive` traits, `#[serde(skip_serializing)]`

---

### Task 1.3: Error handling

**Yêu cầu:**
- Shared `AppError` enum với các variants: `NotFound`, `BadRequest`, `Unauthorized`, `Internal`, `Database`
- Impl `IntoResponse` cho Axum (trả đúng HTTP status code + JSON body `{"error": "..."}`)
- Impl `From<sqlx::Error>` để dùng `?` operator

**Output mong đợi:**
- `cargo build -p common` không error
- Sau khi có server: request không tìm thấy resource → `{"error": "..."}` với status 404

**Rust concept học được:** `enum`, `match`, `impl Trait for Type`, `From` trait

---

### Task 1.4: Redis connection pool

**Yêu cầu:**
- Tạo Redis pool (dùng `deadpool-redis`) từ `REDIS_URL`
- Export từ `common` crate để các crates khác dùng

**Output mong đợi:**
- `cargo build --workspace` không error
- Ping Redis từ code thành công

---

## Phase 2 — API Server: Auth (3-4 ngày)

### Task 2.1: Axum server skeleton

**Yêu cầu:**
- Server khởi động, lắng nghe trên `API_PORT`
- Shared `AppState` chứa db pool, redis pool, config — wrap trong `Arc`
- `GET /health` endpoint không cần auth
- Auto-run migrations khi khởi động
- Init logging với `tracing_subscriber`

**Output mong đợi:**
- `cargo run -p api-server` → log "listening on 0.0.0.0:8080"
- `curl http://localhost:8080/health` → `{"status":"ok"}`

**Rust concept học được:** `async fn`, `Arc` (shared ownership), `Router`, `#[tokio::main]`

---

### Task 2.2: Register endpoint

**Yêu cầu:**
- `POST /auth/register` nhận `{ email, username, password }`
- Hash password bằng Argon2 trước khi lưu DB
- Trả về user info (không có password_hash)

**Output mong đợi:**
- Register thành công → HTTP 200/201, response có `id`, `email`, `username`
- Duplicate email → HTTP 4xx với error message rõ ràng
- Password không được lưu plaintext trong DB

---

### Task 2.3: Login endpoint

**Yêu cầu:**
- `POST /auth/login` nhận `{ email, password }`
- Verify password với Argon2
- Tạo JWT token với payload chứa `user_id`, expiry 24h

**Output mong đợi:**
- Login đúng credentials → HTTP 200, `{"token": "eyJ..."}`
- Login sai password → HTTP 401, `{"error": "Invalid credentials"}`
- Email không tồn tại → HTTP 401 (cùng message, không reveal thông tin)

---

### Task 2.4: JWT auth middleware

**Yêu cầu:**
- Tower middleware extract JWT từ `Authorization: Bearer <token>` header
- Validate token, inject `user_id` (UUID) vào request extensions
- Áp dụng cho các protected routes

**Output mong đợi:**
- Request không có token → 401 `{"error": "Missing token"}`
- Token invalid/expired → 401 `{"error": "Invalid token"}`
- Token hợp lệ → request tiếp tục xử lý bình thường

**Rust concept học được:** Axum middleware, `Extension` extractor, Tower layers

---

### Task 2.5: Integration tests cho auth

**Yêu cầu:**
- Test các scenario: register → login, login sai password, duplicate email
- Test middleware: có token, không có token, token sai

**Output mong đợi:**
- `cargo test -p api-server` → tất cả tests pass, 0 failed

---

## Phase 3 — API Server: Story & Features (4-5 ngày)

### Task 3.1: CRUD Stories

**Yêu cầu:**
- `GET /stories` — phân trang (`page`, `per_page`), trả `{ data, total }`
- `GET /stories/:id` — story detail
- `POST /stories` — [auth required], tạo story mới
- `PUT /stories/:id` — [auth required], chỉ author mới được update
- `DELETE /stories/:id` — [auth required], chỉ author mới được xóa

**Output mong đợi:**
- CRUD hoạt động đúng với đủ HTTP status codes (200, 201, 204, 403, 404)
- Pagination đúng: page 2 trả đúng items, `total` đúng tổng số records

---

### Task 3.2: CRUD Chapters

**Yêu cầu:**
- `GET /stories/:id/chapters` — list chapters của story, sort theo `chapter_order`
- `GET /chapters/:id` — chapter detail + nội dung
- `POST /stories/:id/chapters` — [auth], tạo chapter → trigger publish message lên Redis (task 3.5)
- `PUT /chapters/:id`, `DELETE /chapters/:id` — [auth]

**Output mong đợi:**
- Chapters được trả đúng thứ tự `chapter_order`
- Tạo chapter thành công → message xuất hiện trong Redis stream

---

### Task 3.3: Bookmarks API

**Yêu cầu:**
- `GET /bookmarks` — [auth], list bookmark của user hiện tại
- `POST /bookmarks` — [auth], bookmark một chapter (idempotent, duplicate không lỗi)
- `DELETE /bookmarks/:id` — [auth], chỉ xóa bookmark của chính mình

**Output mong đợi:**
- Bookmark đúng user, không thấy bookmark của user khác
- Duplicate bookmark → không tạo record mới, không error

---

### Task 3.4: Reading History API

**Yêu cầu:**
- `POST /history` — [auth], ghi lại chapter đã đọc (timestamp hiện tại)
- `GET /history` — [auth], list history của user, sort theo `read_at` DESC

**Output mong đợi:**
- History đúng user, sort mới nhất lên đầu
- Cùng chapter đọc nhiều lần → nhiều records (tracking đầy đủ)

---

### Task 3.5: Publish lên Redis Streams

**Yêu cầu:**
- Khi tạo chapter thành công, publish message lên Redis Stream `chapter:audio`
- Message chứa: `chapter_id`, `story_id`, `text` (nội dung chapter)
- Publish bất đồng bộ, không block response API

**Output mong đợi:**
- Sau khi tạo chapter: `redis-cli XRANGE chapter:audio - +` thấy message mới
- API response không chậm hơn đáng kể so với không publish

---

### Task 3.6: Redis caching

**Yêu cầu:**
- Cache story detail với TTL 5 phút
- Cache miss → query DB → set cache
- Update/delete story → invalidate cache tương ứng

**Output mong đợi:**
- Lần 1 (cache miss): response bình thường, `redis-cli GET story:<id>` thấy data
- Lần 2 (cache hit): response nhanh hơn đáng kể, không có query DB log
- Update story → `redis-cli GET story:<id>` → `(nil)`

---

## Phase 4 — Audio Worker (4-6 ngày)

### Task 4.1: Worker binary + graceful shutdown

**Yêu cầu:**
- Binary riêng (`audio-worker`), khởi động bình thường
- Handle `Ctrl+C` (SIGINT) gracefully — không panic, log thông báo rồi thoát

**Output mong đợi:**
- `cargo run -p audio-worker` → log "Audio worker started", chạy liên tục
- Ctrl+C → log "Shutting down gracefully...", process thoát sạch (exit code 0)

**Rust concept học được:** `tokio::select!`, signal handling, graceful shutdown pattern

---

### Task 4.2: Redis Streams consumer (XREADGROUP)

**Yêu cầu:**
- Tạo consumer group `audio-workers` trên stream `chapter:audio`
- `XREADGROUP` blocking (5 giây timeout) để nhận message
- Parse `chapter_id` và `text` từ message fields
- `XACK` sau khi xử lý xong

**Output mong đợi:**
- Worker đang chạy, push message thủ công vào Redis → worker log "Received message: chapter_id=..."
- Message được ACK (không bị reprocess khi restart)

**Rust concept học được:** Redis Streams, consumer groups, `XREADGROUP`, `XACK`

---

### Task 4.3: TTS API integration

**Yêu cầu:**
- Gọi TTS API (OpenAI hoặc alternative) với text content
- Trả về audio bytes (MP3)
- `OPENAI_API_KEY` lấy từ config/env

**Output mong đợi:**
- Gọi TTS với text ngắn → nhận được bytes > 0
- File MP3 play được bằng audio player

---

### Task 4.4: Upload to MinIO

**Yêu cầu:**
- Upload audio bytes lên MinIO bucket `story-audio`
- Key theo pattern `audio/<chapter_id>.mp3`
- Content-Type: `audio/mpeg`
- Trả về file path/key sau khi upload thành công

**Output mong đợi:**
- MinIO Console (localhost:9001) → bucket `story-audio` → thấy file đúng key
- File download được và play được

---

### Task 4.5: Update DB sau upload

**Yêu cầu:**
- Sau khi upload thành công: update `audio_files` record với `status = 'ready'`, `file_path`, `file_size`, `duration_seconds`
- Nếu thất bại: update `status = 'failed'`, `error_message`

**Output mong đợi:**
- Query `audio_files` sau khi worker xử lý xong: `status = 'ready'`, `file_path` có giá trị đúng
- Xử lý thất bại: `status = 'failed'`, `error_message` không null

---

### Task 4.6: Retry logic

**Yêu cầu:**
- TTS call và upload đều có retry với exponential backoff
- Tối đa 3 lần retry
- Log mỗi lần retry (lần thứ mấy, error là gì)

**Output mong đợi:**
- Giả lập TTS thất bại → log "retrying (1/3)...", "retrying (2/3)..."
- Hết retry → log error rõ ràng, cập nhật DB `status = 'failed'`
- Bật lại → xử lý bình thường, không retry nếu thành công lần đầu

---

## Phase 5 — Audio Streaming Service (4-5 ngày)

### Task 5.1: Axum server cho streaming (port 8081)

**Yêu cầu:**
- Server riêng `audio-stream`, lắng nghe trên `STREAM_PORT` (8081)
- `GET /health` endpoint

**Output mong đợi:**
- `cargo run -p audio-stream` → "Audio stream server listening on 0.0.0.0:8081"
- `curl http://localhost:8081/health` → `{"status":"ok"}`

---

### Task 5.2: Full file stream

**Yêu cầu:**
- `GET /audio/:chapter_id` — lấy file từ MinIO, stream về client
- Response header `Content-Type: audio/mpeg`
- File không tồn tại → 404

**Output mong đợi:**
- `curl -o test.mp3 http://localhost:8081/audio/<chapter_id>` → file play được
- Response header có `Content-Type: audio/mpeg`
- Chapter không có audio → 404

---

### Task 5.3: HTTP Range Requests (206 Partial Content)

**Yêu cầu:**
- Support `Range: bytes=X-Y` header
- Response với `206 Partial Content`, `Content-Range`, `Content-Length` đúng

**Output mong đợi:**
- `curl -H "Range: bytes=0-1023" ...` → HTTP 206, `Content-Range: bytes 0-1023/<total>`, `Content-Length: 1024`
- Browser audio player seek/skip hoạt động không bị stall

---

### Task 5.4: Memory-efficient streaming

**Yêu cầu:**
- Stream file theo chunks, không load toàn bộ file vào RAM
- Dùng `ReaderStream` hoặc tương đương

**Output mong đợi:**
- Stream file 50MB: memory usage của process không tăng thêm 50MB
- `ps aux` hoặc Activity Monitor: RSS ổn định ~20-30MB khi streaming

---

### Task 5.5: Concurrent streaming test

**Yêu cầu:**
- Test với 100 concurrent connections
- Không có request failure

**Output mong đợi:**
- `wrk -t4 -c100 -d10s <url>` hoặc `k6` → Requests/sec > 500, 0 errors, avg latency < 200ms

---

### Task 5.6: CORS middleware

**Yêu cầu:**
- Allow origin từ React dev server (localhost:5173) và production domain
- Handle OPTIONS preflight request

**Output mong đợi:**
- Preflight request → response có `Access-Control-Allow-Origin`, `Access-Control-Allow-Methods`
- Audio player trên React (localhost:5173) không bị CORS error trong browser Console

---

## Phase 6 — React Frontend (3-5 ngày)

### Task 6.1: Setup Vite + React + Ant Design

**Yêu cầu:**
- Vite + React + TypeScript
- Ant Design cho UI components
- React Query cho data fetching
- React Router cho navigation
- Axios client với base URL từ env

**Output mong đợi:**
- `npm run dev` → trang load được tại localhost:5173, không có console error
- Layout cơ bản với Ant Design đã áp dụng

---

### Task 6.2: Story List page

**Yêu cầu:**
- Route `/` hoặc `/stories` — hiển thị danh sách stories từ API
- Pagination
- Click vào story → navigate sang trang detail

**Output mong đợi:**
- Stories hiển thị với title, description, cover image (nếu có)
- Next/prev page hoạt động, URL có query params `?page=X`
- Click story → redirect đúng trang

---

### Task 6.3: Chapter Reader page

**Yêu cầu:**
- Route `/stories/:storyId/chapters/:chapterId`
- Hiển thị nội dung chapter
- Nút Next/Prev chapter
- Auto-mark as read khi scroll đến cuối trang

**Output mong đợi:**
- Nội dung chapter hiển thị đầy đủ, đúng formatting
- Next/Prev chapter navigate đúng
- Sau khi scroll hết → record xuất hiện trong reading history API

---

### Task 6.4: Audio Player component

**Yêu cầu:**
- Hiển thị khi chapter có audio (status = ready)
- Play/Pause, seek bar, progress bar, thời gian hiện tại / tổng thời gian
- Stream từ `audio-stream` service (port 8081)

**Output mong đợi:**
- Play/Pause hoạt động
- Seek bar kéo được, audio nhảy đúng vị trí
- Progress bar update theo thời gian
- Không có CORS error trong browser Console

---

### Task 6.5: Bookmarks + History

**Yêu cầu:**
- Nút bookmark trong chapter reader
- Route `/bookmarks` — list chapters đã bookmark
- Route `/history` — list chapters đã đọc, sort mới nhất trước

**Output mong đợi:**
- Click bookmark → icon đổi trạng thái ngay (optimistic update hoặc refetch)
- `/bookmarks` hiển thị đúng danh sách
- `/history` sort đúng theo thời gian

---

### Task 6.6: Auth pages + protected routes

**Yêu cầu:**
- Route `/login`, `/register` với form validation cơ bản
- Protected routes: redirect về `/login` nếu chưa đăng nhập
- Lưu JWT token trong localStorage
- JWT hết hạn → auto redirect về `/login`

**Output mong đợi:**
- Login thành công → redirect về trang chủ
- Truy cập `/bookmarks` khi chưa login → redirect `/login`
- Token hết hạn → API trả 401 → app redirect về `/login`

---

## Phase 7 — Polish & Deploy (2-3 ngày)

### Task 7.1: Dockerfile (multi-stage build)

**Yêu cầu:**
- Multi-stage: build stage (rust image) + runtime stage (debian slim)
- Riêng Dockerfile cho `api-server`, `audio-worker`, `audio-stream`
- Image size tối thiểu

**Output mong đợi:**
- `docker build` thành công
- Image size `api-server` < 50MB
- Container chạy được, không missing shared library

---

### Task 7.2: Docker Compose full stack

**Yêu cầu:**
- Compose file chạy toàn bộ: postgres, redis, minio, api-server, audio-worker, audio-stream
- Service order đúng (infra trước, app sau)
- Health checks và `depends_on`

**Output mong đợi:**
- `docker compose up --build -d` → 6 services running, không có port conflict
- `curl http://localhost:8080/health` và `curl http://localhost:8081/health` → cả hai OK

---

### Task 7.3: README

**Yêu cầu:**
- Hướng dẫn setup từ đầu: prerequisites, clone, run
- Giải thích kiến trúc ngắn gọn
- Các lệnh dev thường dùng

**Output mong đợi:**
- Người mới clone repo, chỉ đọc README → setup và chạy được trong 10 phút

---

### Task 7.4: Seed data

**Yêu cầu:**
- Script hoặc lệnh tạo dữ liệu mẫu: 3-5 stories, mỗi story vài chapters, 1-2 users

**Output mong đợi:**
- Sau khi chạy seed: `GET /stories` trả ít nhất 3 stories
- Stories có chapters, có thể test full flow mà không cần tự tạo data

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
