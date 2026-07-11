# Keamanan — wa-sync

Ringkasan kontrol keamanan yang sudah ada, dan yang perlu dilakukan sebelum produksi.

## Sudah ada
- **Auth admin**: bcrypt (`$2y$12$`), cek `banned_at`/`is_active`, CSRF token per-sesi, sesi persisten di Postgres (`rust_sessions`).
- **Rate-limit login**: lockout bertingkat (`ratelimit.rs`) mencegah brute-force.
- **Rate-limit API publik**: 60 permintaan/menit per `appkey` (`publicapi::rate_ok`).
- **RBAC**: `CurrentUser` + permission (Spatie), Superadmin bypass; halaman admin di-gate `view_resources`.
- **Endpoint internal sidecar** (`/internal/wa/*`): dijaga `X-WA-Secret` (bandingkan constant-time `subtle`) + cek loopback.
- **Webhook Midtrans** (`/api/billing/*/notification`): verifikasi signature SHA512 constant-time, idempoten, anti-tamper harga (harga dari DB).
- **Webhook keluar**: HMAC-SHA256 (`X-WA-Signature`).
- **Header keamanan**: `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy` di semua respons.
- **Secret tidak di repo**: `.env` & `wa-sidecar/auth/` di-`.gitignore`.

## Perlu sebelum produksi (rekomendasi)
1. **HTTPS/TLS** — jalankan di belakang reverse proxy (nginx/Caddy) dengan TLS. Saat ini HTTP lokal.
2. **2FA admin (TOTP)** — DITUNDA: butuh dependency TOTP (mis. `totp-rs`) + kolom `users.totp_secret` + alur verifikasi saat login + QR untuk authenticator. Prioritaskan untuk akun Superadmin.
3. **Enkripsi secret at-rest** — DITUNDA: `midtrans_server_key` (settings) sebaiknya dienkripsi dengan master key dari env (mis. `aes-gcm`). Catatan: `auth_key`/`app_key` bersifat token yang di-lookup by-value (seperti API key) sehingga tak bisa dienkripsi non-deterministik; pertimbangkan hashing + prefix bila mau lebih ketat.
4. **Rotasi kredensial** — DB password, Gmail app password, `WA_SHARED_SECRET` sebaiknya dirotasi berkala & disimpan di secret manager, bukan `.env` datar.
5. **Backup terenkripsi** — `backup.ps1` menghasilkan dump + zip auth; simpan di lokasi aman/terenkripsi (berisi kredensial WA).
6. **Audit & monitoring** — `activity_log` mencatat aksi admin; tambahkan alerting uptime nomor (email banned/logout sudah ada) + monitoring error (mis. Sentry).

## Testing
- Unit test: `cargo test` (broadcast spintax/variabel, signature Midtrans, inferensi media, rate-limit).
- CI: `.github/workflows/ci.yml` menjalankan `cargo check`+`cargo test` & `node --check` tiap push/PR.
- E2E: pakai **mode simulasi** (nomor sim) untuk uji terima/AI/broadcast tanpa nomor nyata; untuk pembayaran pakai sandbox Midtrans + ngrok.
