# dev.ps1 — jalankan server Rust wa-sync (central)
# Memastikan PATH berisi cargo + WinLibs MinGW (dlltool/gcc), lalu cargo run.
# Pakai: dari folder ini jalankan  ->  .\dev.ps1
$env:Path = "$env:USERPROFILE\.cargo\bin;" +
    [Environment]::GetEnvironmentVariable('Path', 'Machine') + ";" +
    [Environment]::GetEnvironmentVariable('Path', 'User')
Set-Location -Path $PSScriptRoot
Write-Host "Menjalankan server Rust wa-sync di http://127.0.0.1:8090 ..." -ForegroundColor Cyan
cargo run
