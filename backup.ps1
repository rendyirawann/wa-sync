# Backup wa-sync: dump database + folder auth sesi WhatsApp (creds Baileys).
# Jalankan manual, atau jadwalkan via Task Scheduler (mis. tiap hari 02:00).
#   powershell -ExecutionPolicy Bypass -File C:\xampp\htdocs\myProject\wa-sync\backup.ps1
# Restore DB: pg_restore -h 127.0.0.1 -p 5433 -U postgres -d wa_sync --clean backups\<stamp>\wa_sync.dump

$ErrorActionPreference = 'Continue'
$root  = $PSScriptRoot
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$dir   = Join-Path $root ("backups\" + $stamp)
New-Item -ItemType Directory -Force -Path $dir | Out-Null

# 1) Dump database Postgres (:5433)
$env:PGPASSWORD = 'in12345'
$pgDump = (Get-Command pg_dump -ErrorAction SilentlyContinue).Source
if (-not $pgDump) {
    $pgDump = (Get-ChildItem 'C:\Program Files\PostgreSQL\*\bin\pg_dump.exe' -ErrorAction SilentlyContinue | Select-Object -First 1).FullName
}
if ($pgDump) {
    & $pgDump -h 127.0.0.1 -p 5433 -U postgres -d wa_sync -F c -f (Join-Path $dir 'wa_sync.dump')
    Write-Host "[OK] Database -> wa_sync.dump"
} else {
    Write-Warning "pg_dump tidak ditemukan. Install PostgreSQL client atau tambahkan bin-nya ke PATH."
}

# 2) Backup folder auth (kredensial sesi WhatsApp) — sangat penting, ini 'login' WA.
$auth = Join-Path $root 'wa-sidecar\auth'
if (Test-Path $auth) {
    Compress-Archive -Path $auth -DestinationPath (Join-Path $dir 'wa-auth.zip') -Force
    Write-Host "[OK] Creds WA -> wa-auth.zip"
} else {
    Write-Host "[skip] wa-sidecar\auth belum ada (belum ada nomor tertaut)."
}

# 3) Rotasi: simpan 14 backup terbaru saja
Get-ChildItem (Join-Path $root 'backups') -Directory -ErrorAction SilentlyContinue |
    Sort-Object Name -Descending | Select-Object -Skip 14 |
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "Backup selesai: $dir"
