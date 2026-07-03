// PM2 process manager — jalankan WA Service tetap hidup walau terminal ditutup,
// auto-restart kalau crash. Pakai untuk "biar online terus" di PC/server yang selalu nyala.
//
// Setup sekali:
//   npm install -g pm2
//   cargo build --release --manifest-path rust/crates/central/Cargo.toml   (bikin central.exe rilis)
//   pm2 start ecosystem.config.js
//   pm2 save
// Survive REBOOT (Windows): npm install -g pm2-windows-startup && pm2-startup install && pm2 save
// Perintah harian: pm2 status | pm2 logs | pm2 restart all | pm2 stop all
module.exports = {
  apps: [
    {
      name: 'wa-central',
      // Pakai binary RILIS (lebih cepat & stabil dari `cargo run`). Build dulu (lihat atas).
      script: './rust/target/release/central.exe',
      interpreter: 'none',
      cwd: __dirname,
      autorestart: true,
      max_restarts: 50,
      restart_delay: 3000,
      // central.exe memuat .env & templates dari path compile-time (rust/crates/central) otomatis.
    },
    {
      name: 'wa-sidecar',
      script: './wa-sidecar/index.js',
      node_args: '--env-file=./wa-sidecar/.env',
      cwd: __dirname,
      autorestart: true,
      max_restarts: 50,
      restart_delay: 3000,
    },
  ],
};
