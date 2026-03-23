# Story Reader Platform — Build Plan

> Go + Rust + React | 38 tasks | ~5 tuần

---

## Phase 0 — Setup & Design `Infra` *(2–3 ngày)*

- [ ] Khởi tạo mono-repo hoặc multi-repo
  - ✓ Output: cấu trúc thư mục `go-services/`, `rust-services/`, `frontend/`, `docker/`
- [ ] Viết `docker-compose.yml` với PostgreSQL, Redis, MinIO
  - ✓ Output: `docker-compose.yml` chạy được, kết nối health check OK
- [ ] Thiết kế database schema (users, stories, chapters, audio_files, reading_history, bookmarks)
  - ✓ Output: file `schema.sql` hoặc migration files, có đủ FK và index cơ bản
- [ ] Vẽ system architecture diagram (draw.io hoặc Excalidraw)
  - ✓ Output: diagram lưu vào `docs/`, thể hiện rõ luồng data giữa các service
- [ ] Định nghĩa API contract (OpenAPI/Swagger cho Go API Gateway)
  - ✓ Output: `openapi.yaml` với đủ endpoints cho stories, chapters, users, auth

---

## Phase 1 — Go: User Service `Go` *(3–4 ngày)*

- [ ] Khởi tạo Go module, cài Gin/Fiber + GORM + PostgreSQL driver
  - ✓ Output: `go.mod`, `main.go` chạy được, connect DB thành công
- [ ] Implement Register endpoint (`POST /auth/register`)
  - ✓ Output: user được lưu vào DB, password hash bằng bcrypt
- [ ] Implement Login endpoint (`POST /auth/login`)
  - ✓ Output: trả về JWT token hợp lệ, có expiry
- [ ] Implement middleware xác thực JWT
  - ✓ Output: middleware từ chối request không có token, attach `user_id` vào context
- [ ] Viết unit test cho auth logic
  - ✓ Output: ít nhất test Register, Login happy path và error cases

---

## Phase 2 — Go: Story Service `Go` *(4–5 ngày)*

- [ ] CRUD Stories (`GET` list, `GET` detail, `POST`, `PUT`, `DELETE`)
  - ✓ Output: 5 endpoints hoạt động, có pagination cho list
- [ ] CRUD Chapters (`GET` list by story, `GET` detail, `POST`, `PUT`, `DELETE`)
  - ✓ Output: chapters liên kết đúng với `story_id`, có sort theo order
- [ ] Implement Bookmark API (`POST /bookmarks`, `DELETE /bookmarks/:id`, `GET /bookmarks`)
  - ✓ Output: user bookmark được chapter, trả về danh sách bookmarks của user
- [ ] Implement Reading History (`POST /history`, `GET /history`)
  - ✓ Output: ghi lại `chapter_id` + timestamp khi user đọc, query được lịch sử
- [ ] Publish message lên Queue khi chapter mới được tạo
  - ✓ Output: message JSON có `chapter_id` được đẩy lên Redis Streams hoặc RabbitMQ
- [ ] Implement Redis caching cho story/chapter detail
  - ✓ Output: hit cache trả về < 5ms, cache invalidation khi update

---

## Phase 3 — Rust: Audio Worker `Rust` *(4–6 ngày)*

- [ ] Khởi tạo Rust project với Cargo, thêm `tokio` + `serde_json` dependencies
  - ✓ Output: `Cargo.toml`, `src/main.rs` với async main chạy được
- [ ] Implement Queue consumer (đọc message từ Redis Streams hoặc RabbitMQ)
  - ✓ Output: worker nhận được message JSON, parse `chapter_id` thành công
- [ ] Integrate TTS API (Google TTS hoặc OpenAI TTS) để generate audio
  - ✓ Output: gọi API với chapter text, nhận về audio bytes (mp3/wav)
- [ ] Upload audio file lên MinIO (S3-compatible)
  - ✓ Output: file được lưu với key `audio/{chapter_id}.mp3`, presigned URL hoạt động
- [ ] Cập nhật `audio_files` table sau khi upload thành công
  - ✓ Output: record trong DB có `file_path`, `duration`, `status = ready`
- [ ] Xử lý retry logic khi TTS hoặc upload thất bại
  - ✓ Output: tối đa 3 lần retry với exponential backoff, log rõ error

---

## Phase 4 — Rust: Audio Streaming Service `Rust` *(5–7 ngày)*

- [ ] Khởi tạo Axum server với tokio runtime
  - ✓ Output: `GET /health` trả về 200 OK, server chạy port 8081
- [ ] Implement `GET /audio/:chapter_id` — full file stream
  - ✓ Output: client nhận được audio file hoàn chỉnh, Content-Type đúng
- [ ] Implement HTTP Range Requests (partial content)
  - ✓ Output: request với `Range` header nhận được `206 Partial Content`, đúng byte range
- [ ] Stream audio từ MinIO (không load toàn bộ file vào RAM)
  - ✓ Output: dùng async byte stream, memory usage ổn định dù file 50MB
- [ ] Test concurrent streaming (dùng k6 hoặc wrk)
  - ✓ Output: 100 concurrent requests không bị drop, latency < 200ms cho first byte
- [ ] Thêm CORS headers cho phép React frontend gọi trực tiếp
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
  - ✓ Output: JWT lưu vào localStorage, redirect nếu chưa login

---

## Phase 6 — Polish & Deploy `Infra` *(2–3 ngày)*

- [ ] Viết Dockerfile cho từng service (Go, Rust ×2, React)
  - ✓ Output: 4 Dockerfile, build thành công, image size hợp lý (Go < 30MB, Rust < 20MB)
- [ ] Cập nhật `docker-compose.yml` để chạy full stack
  - ✓ Output: `docker compose up --build` khởi động toàn bộ hệ thống, không conflict port
- [ ] Viết README với architecture diagram, setup guide, API docs
  - ✓ Output: `README.md` đủ để người khác clone về và chạy trong 10 phút
- [ ] Seed data demo (stories + chapters) để showcase
  - ✓ Output: script seed tạo 5 stories, mỗi story 3 chapters, 1 audio file mẫu

---

*38 tasks tổng · ~5 tuần với AI coding assistant*
