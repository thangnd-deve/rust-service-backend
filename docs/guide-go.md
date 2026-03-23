# Story Reader Platform — Build Plan (Full Go)

> Go + React | 36 tasks | ~4 tuần

## Tech Stack

- **Backend**: Gin/Fiber + GORM + PostgreSQL
- **Audio Worker**: Go routine + Redis Streams
- **Audio Streaming**: net/http hoặc Fiber với io.Reader streaming
- **DB**: PostgreSQL (GORM)
- **Cache**: Redis (`go-redis/v9`)
- **Queue**: Redis Streams
- **Storage**: MinIO (`minio-go/v7`)
- **Auth**: `golang-jwt/v5` + `bcrypt`
- **Frontend**: Vite + React + TypeScript + Ant Design + TanStack Query

## Project Layout

```text
story-reader/
├── cmd/
│   ├── api/             # main API server
│   ├── audio-worker/    # queue consumer + TTS
│   └── audio-stream/    # audio streaming server
├── internal/
│   ├── config/          # env config
│   ├── models/          # GORM models
│   ├── handlers/        # HTTP handlers
│   ├── middleware/       # JWT auth, CORS, logging
│   ├── repository/      # DB queries
│   ├── service/         # business logic
│   ├── cache/           # Redis helpers
│   └── queue/           # Redis Streams producer/consumer
├── migrations/          # SQL migration files
├── frontend/            # React app
├── docker/
└── go.mod
```

---

## Phase 0 — Setup & Design `Infra` *(2–3 ngày)*

- [ ] Khởi tạo Go module, setup project layout theo standard Go layout
  - ✓ Output: `go.mod`, thư mục `cmd/`, `internal/`, `go build ./...` thành công
- [ ] Viết `docker-compose.yml` với PostgreSQL, Redis, MinIO
  - ✓ Output: `docker-compose.yml` chạy được, kết nối health check OK
- [ ] Thiết kế database schema (users, stories, chapters, audio_files, reading_history, bookmarks)
  - ✓ Output: migration files trong `migrations/`, có đủ FK và index cơ bản
- [ ] Vẽ system architecture diagram (draw.io hoặc Excalidraw)
  - ✓ Output: diagram lưu vào `docs/`, thể hiện rõ luồng data giữa các service
- [ ] Định nghĩa API contract (OpenAPI/Swagger)
  - ✓ Output: `openapi.yaml` hoặc dùng `swaggo/swag` auto-generate từ annotations

---

## Phase 1 — API Server: Auth `Go` *(2–3 ngày)*

- [ ] Khởi tạo Gin/Fiber server, setup router + DB connection (GORM)
  - ✓ Output: `GET /health` trả về 200, server chạy port 8080, DB connect OK
- [ ] Implement Register endpoint (`POST /auth/register`)
  - ✓ Output: user lưu vào DB, password hash bằng `bcrypt`
- [ ] Implement Login endpoint (`POST /auth/login`)
  - ✓ Output: trả về JWT token hợp lệ, có expiry
- [ ] Implement JWT auth middleware
  - ✓ Output: middleware reject request không có token, set `user_id` vào gin.Context
- [ ] Viết unit test cho auth logic
  - ✓ Output: test Register, Login happy path và error cases

---

## Phase 2 — API Server: Story & Features `Go` *(3–4 ngày)*

- [ ] CRUD Stories (`GET` list, `GET` detail, `POST`, `PUT`, `DELETE`)
  - ✓ Output: 5 endpoints hoạt động, pagination với `page` + `page_size` params
- [ ] CRUD Chapters (`GET` list by story, `GET` detail, `POST`, `PUT`, `DELETE`)
  - ✓ Output: chapters liên kết đúng với `story_id`, sort theo `order` field
- [ ] Implement Bookmark API (`POST /bookmarks`, `DELETE /bookmarks/:id`, `GET /bookmarks`)
  - ✓ Output: user bookmark chapter, trả về danh sách bookmarks
- [ ] Implement Reading History (`POST /history`, `GET /history`)
  - ✓ Output: ghi lại `chapter_id` + timestamp, query lịch sử theo user
- [ ] Publish message lên Redis Streams khi chapter mới được tạo
  - ✓ Output: message JSON `{chapter_id, story_id}` xuất hiện trong stream
- [ ] Implement Redis caching cho story/chapter detail
  - ✓ Output: cache hit < 5ms, cache invalidation khi update/delete

---

## Phase 3 — Audio Worker `Go` *(3–4 ngày)*

- [ ] Khởi tạo `audio-worker` cmd, setup goroutine pool + graceful shutdown
  - ✓ Output: worker start, connect Redis + DB + MinIO thành công
- [ ] Implement Redis Streams consumer (XREADGROUP)
  - ✓ Output: worker nhận message JSON, parse `chapter_id` thành công
- [ ] Integrate TTS API (Google TTS hoặc OpenAI TTS) qua `net/http`
  - ✓ Output: gọi API với chapter text, nhận về audio bytes (mp3)
- [ ] Upload audio file lên MinIO (dùng `minio-go/v7`)
  - ✓ Output: file lưu với key `audio/{chapter_id}.mp3`, presigned URL hoạt động
- [ ] Cập nhật `audio_files` table sau khi upload thành công
  - ✓ Output: record trong DB có `file_path`, `duration`, `status = ready`
- [ ] Xử lý retry logic với exponential backoff
  - ✓ Output: tối đa 3 lần retry, log rõ error qua `zerolog` hoặc `zap`

---

## Phase 4 — Audio Streaming Service `Go` *(3–4 ngày)*

- [ ] Khởi tạo HTTP server cho audio streaming
  - ✓ Output: `GET /health` trả về 200 OK, server chạy port 8081
- [ ] Implement `GET /audio/:chapter_id` — full file stream
  - ✓ Output: client nhận được audio file hoàn chỉnh, Content-Type đúng
- [ ] Implement HTTP Range Requests (partial content — `206 Partial Content`)
  - ✓ Output: request với `Range` header nhận đúng byte range
  - Tip: Go stdlib `http.ServeContent` hỗ trợ Range sẵn
- [ ] Stream audio từ MinIO qua `io.Reader` (không load toàn bộ vào RAM)
  - ✓ Output: dùng `io.Copy` với `GetObject`, memory usage ổn định
- [ ] Test concurrent streaming (k6 hoặc wrk)
  - ✓ Output: 100 concurrent requests không drop, latency < 200ms cho first byte
- [ ] CORS middleware
  - ✓ Output: browser không bị CORS error khi play audio từ React

---

## Phase 5 — React Frontend `React` *(3–5 ngày)*

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
  - ✓ Output: JWT lưu vào httpOnly cookie, redirect nếu chưa login

---

## Phase 6 — Polish & Deploy `Infra` *(2–3 ngày)*

- [ ] Viết Dockerfile cho từng service (3 Go binaries, 1 React)
  - ✓ Output: multi-stage build, image size < 30MB mỗi Go binary
- [ ] Cập nhật `docker-compose.yml` để chạy full stack
  - ✓ Output: `docker compose up --build` khởi động toàn bộ, không conflict port
- [ ] Viết README với architecture diagram, setup guide, API docs
  - ✓ Output: `README.md` đủ để người khác clone về và chạy trong 10 phút
- [ ] Seed data demo (stories + chapters) để showcase
  - ✓ Output: script seed tạo 5 stories, mỗi story 3 chapters, 1 audio file mẫu

---

## Key Go Libraries

| Mục đích | Library |
|---|---|
| Web framework | `gin-gonic/gin` hoặc `gofiber/fiber` |
| ORM | `gorm.io/gorm` |
| Redis | `redis/go-redis/v9` |
| MinIO/S3 | `minio/minio-go/v7` |
| JWT | `golang-jwt/jwt/v5` |
| Password hash | `golang.org/x/crypto/bcrypt` |
| HTTP client | `net/http` (stdlib) |
| Logging | `rs/zerolog` hoặc `uber-go/zap` |
| Config | `spf13/viper` hoặc `joho/godotenv` |
| Swagger | `swaggo/swag` |
| Testing | `stretchr/testify` + `testcontainers-go` |

---

*36 tasks tổng · ~4 tuần với AI coding assistant*
*Lợi thế: dev speed nhanh, compile < 5s, ecosystem mature, dễ hire/onboard*
