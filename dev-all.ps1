# dev-all.ps1 — jalankan server Rust (central) + sidecar WhatsApp (Baileys) bersamaan.
# Ctrl+C menghentikan keduanya. Pakai: dari root project jalankan  ->  .\dev-all.ps1
$env:Path = "$env:USERPROFILE\.cargo\bin;" +
    [Environment]::GetEnvironmentVariable('Path', 'Machine') + ";" +
    [Environment]::GetEnvironmentVariable('Path', 'User')
Set-Location -Path $PSScriptRoot
Write-Host "Menjalankan Rust (:8090) + sidecar WA (:8099) ..." -ForegroundColor Cyan
& "node_modules\.bin\concurrently" -n "rust,wa" -c "green,magenta" `
    "cargo run --manifest-path rust/crates/central/Cargo.toml" `
    "node --env-file=wa-sidecar/.env wa-sidecar/index.js"
