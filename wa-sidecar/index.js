// WhatsApp gateway sidecar (Baileys) untuk wa-sync.
// Dipanggil Rust/Axum lewat loopback HTTP (header X-WA-Secret), dan mengirim event
// balik ke Rust via POST {RUST_URL}/internal/wa/event. Satu proses = banyak sesi.

import Fastify from 'fastify';
import pino from 'pino';
import qrcode from 'qrcode';
import { timingSafeEqual } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import fs from 'node:fs/promises';
import * as Baileys from '@whiskeysockets/baileys';

// --- Baileys 7 = ESM murni: ambil named export lewat namespace import ---
const makeWASocket = Baileys.makeWASocket || Baileys.default;
const { useMultiFileAuthState, DisconnectReason, fetchLatestBaileysVersion } = Baileys;

const PORT = Number(process.env.PORT || 8099);
const RUST_URL = process.env.RUST_URL || 'http://127.0.0.1:8090';
const SECRET = process.env.WA_SHARED_SECRET || '';
const HERE = path.dirname(fileURLToPath(import.meta.url));
const authDir = (id) => path.join(HERE, 'auth', String(id).replace(/[^a-zA-Z0-9_-]/g, ''));

if (!SECRET) console.warn('[wa-sidecar] WA_SHARED_SECRET kosong — endpoint tidak aman!');

/** sessions: id -> { sock, status, qr, phone, jid, retries, starting, q, pumping, hWin, hCnt, dWin, dCnt } */
const sessions = new Map();

// --- Konfigurasi anti-ban (override via .env) ---
const DRY = process.env.WA_DRY_RUN === '1';            // mode uji: tanpa socket WA nyata
const num = (v, d) => { const n = parseInt(v, 10); return Number.isFinite(n) ? n : d; };
// Level anti-ban per-NOMOR (dipilih user). 'normal' = nilai .env (kompatibel lama).
const LEVELS = {
  safe:       { minGap: 12000, maxGap: 30000, typMin: 1500, typMax: 4000, hourCap: 20, dayCap: 120 },
  normal:     { minGap: num(process.env.WA_MIN_GAP_MS, 6000), maxGap: num(process.env.WA_MAX_GAP_MS, 18000), typMin: num(process.env.WA_TYPING_MIN_MS, 1000), typMax: num(process.env.WA_TYPING_MAX_MS, 3000), hourCap: num(process.env.WA_HOURLY_CAP, 40), dayCap: num(process.env.WA_DAILY_CAP, 300) },
  aggressive: { minGap: 3000, maxGap: 9000, typMin: 600, typMax: 1500, hourCap: 80, dayCap: 600 },
};
const cfgFor = (lvl) => LEVELS[lvl] || LEVELS.normal;
const rand = (a, b) => a + Math.floor(Math.random() * (b - a + 1));
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
function rollWindows(s) {
  const t = Date.now();
  if (!s.hWin || t - s.hWin >= 3600000) { s.hWin = t; s.hCnt = 0; }
  if (!s.dWin || t - s.dWin >= 86400000) { s.dWin = t; s.dCnt = 0; }
}
function capStats(s) {
  rollWindows(s);
  const c = s.cfg || cfgFor('normal');
  return { hourLeft: Math.max(0, c.hourCap - (s.hCnt || 0)), dayLeft: Math.max(0, c.dayCap - (s.dCnt || 0)), queue: (s.q || []).length, level: s.level || 'normal' };
}

// --- kirim event ke Rust (fire-and-forget) ---
function postEvent(id, type, data) {
  fetch(`${RUST_URL}/internal/wa/event`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'X-WA-Secret': SECRET },
    body: JSON.stringify({ session_id: id, type, data: data || {} }),
  }).catch((e) => app.log.warn(`postEvent(${type}) gagal: ${e.message}`));
}

async function wipeAuth(id) {
  await fs.rm(authDir(id), { recursive: true, force: true }).catch(() => {});
}

// --- ambil counter cap dari DB (Rust) supaya cap PERSISTEN melintasi restart sidecar ---
async function fetchUsage(id) {
  try {
    const r = await fetch(`${RUST_URL}/internal/wa/usage/${id}`, { headers: { 'X-WA-Secret': SECRET } });
    if (!r.ok) return null;
    return await r.json();
  } catch { return null; }
}
async function seedCounters(s, id) {
  const u = await fetchUsage(id);
  const t = Date.now();
  s.hWin = t; s.hCnt = (u && u.hour_sent) || 0;
  s.dWin = t; s.dCnt = (u && u.day_sent) || 0;
}

// --- lifecycle satu sesi ---
async function startSession(id, level, sim) {
  // Mode uji / SIMULASI: sesi "tersambung" palsu tanpa Baileys nyata (global DRY atau per-sesi sim).
  if (DRY || sim) {
    const d = sessions.get(id) || {};
    d.status = 'connected'; d.q = d.q || [];
    d.sim = !!sim;
    d.level = level || d.level || 'normal'; d.cfg = cfgFor(d.level);
    sessions.set(id, d);
    await seedCounters(d, id);
    postEvent(id, 'connected', { phone: sim ? '628000SIM' + String(id).replace(/\D/g, '').slice(0, 4) : '620000DRY', jid: (sim ? 'sim' : 'dry') + '@s.whatsapp.net' });
    if (d.q.length) pump(id).catch(() => {});
    return;
  }
  const cur = sessions.get(id);
  if (cur && (cur.status === 'connected' || cur.starting)) return; // idempotent
  const s = cur || {};
  s.starting = true;
  s.level = level || s.level || 'normal'; s.cfg = cfgFor(s.level);
  await seedCounters(s, id);
  s.status = s.status === 'connected' ? s.status : 'connecting';
  sessions.set(id, s);

  // Tutup socket LAMA (bila ada) + lepas listener-nya sebelum buat yang baru.
  // Tanpa ini, saat reconnect ada DUA socket aktif yang saling tendang → loop "qr ulang".
  if (s.sock) {
    try { s.sock.ev.removeAllListeners(); } catch { /* ignore */ }
    try { s.sock.end(undefined); } catch { /* ignore */ }
    s.sock = null;
  }

  const { state, saveCreds } = await useMultiFileAuthState(authDir(id));
  let version;
  try { ({ version } = await fetchLatestBaileysVersion()); } catch { /* pakai default */ }

  const sock = makeWASocket({
    version,
    auth: state,
    logger: pino({ level: 'silent' }),
    browser: ['WA Service', 'Chrome', '121.0'],
    syncFullHistory: false,
    markOnlineOnConnect: false,
    keepAliveIntervalMs: 25000,   // ping berkala → koneksi lebih stabil
    connectTimeoutMs: 60000,
    retryRequestDelayMs: 2000,
  });
  s.sock = sock;
  s.starting = false;
  sessions.set(id, s);

  sock.ev.on('creds.update', saveCreds);
  sock.ev.on('connection.update', (u) => onConn(id, u).catch((e) => app.log.error(e.message)));
  sock.ev.on('messages.upsert', (m) => onMsg(id, m).catch((e) => app.log.error(e.message)));
}

async function onConn(id, u) {
  const s = sessions.get(id);
  if (!s) return;
  const { connection, lastDisconnect, qr } = u;

  if (qr) {
    s.status = 'qr';
    s.qr = await qrcode.toDataURL(qr).catch(() => null);
    postEvent(id, 'qr', { qr: s.qr });
  }
  if (connection === 'open') {
    s.status = 'connected';
    s.qr = null;
    s.retries = 0;
    const jid = sock_user_jid(s.sock);
    s.jid = jid;
    s.phone = jid ? jid.split('@')[0].split(':')[0] : null;
    postEvent(id, 'connected', { phone: s.phone, jid });
    if (s.q && s.q.length) pump(id).catch(() => {}); // lanjutkan antrian setelah reconnect
  }
  if (connection === 'close') {
    const code = lastDisconnect?.error?.output?.statusCode;
    s.starting = false;
    if (code === DisconnectReason.loggedOut) {
      // 401: perangkat dicabut dari HP → butuh scan ulang
      s.status = 'logged_out';
      await wipeAuth(id);
      sessions.delete(id);
      postEvent(id, 'logged_out', { reason: 'loggedOut' });
    } else if (code === DisconnectReason.restartRequired) {
      // 515: normal sesaat setelah pairing → sambung lagi cepat (BUKAN error)
      app.log.info(`session ${id}: restart required (515) → reconnect`);
      setTimeout(() => startSession(id).catch(() => {}), 800);
    } else if (code === DisconnectReason.connectionReplaced || code === DisconnectReason.forbidden) {
      // 440/403: sesi diambil alih perangkat lain (mis. WhatsApp Web nomor sama) → JANGAN saling tendang
      s.status = 'disconnected';
      app.log.warn(`session ${id}: connection replaced/forbidden (${code}) → stop auto-reconnect`);
      postEvent(id, 'disconnected', { reason: 'replaced', note: 'Nomor sedang dipakai di perangkat/WhatsApp Web lain. Tutup sesi lain itu, lalu klik reload.' });
    } else {
      // 428/408/lainnya: koneksi putus sementara → reconnect backoff
      s.status = 'disconnected';
      postEvent(id, 'disconnected', { reason: String(code || 'unknown') });
      scheduleReconnect(id);
    }
  }
}

function sock_user_jid(sock) {
  try { return sock?.user?.id || null; } catch { return null; }
}

function scheduleReconnect(id) {
  const s = sessions.get(id);
  if (!s) return;
  s.retries = (s.retries || 0) + 1;
  if (s.retries > 10) return; // berhenti setelah 10x
  const delay = Math.min(30000, 1000 * 2 ** s.retries);
  setTimeout(() => startSession(id).catch(() => {}), delay);
}

// Resolusi alamat pengirim ke NOMOR ASLI (@s.whatsapp.net) meski WhatsApp mengirim
// sebagai @lid (privasi nomor disembunyikan). Tanpa ini, balasan tidak sampai ke kontak.
async function resolvePn(sock, key) {
  const raw = key?.remoteJid || '';
  const alt = key?.remoteJidAlt || '';
  if (raw.endsWith('@s.whatsapp.net')) return raw;
  if (alt && alt.endsWith('@s.whatsapp.net')) return alt; // WA sertakan PN di field alt
  if (raw.endsWith('@lid')) {
    try {
      const pn = await sock?.signalRepository?.lidMapping?.getPNForLID?.(raw);
      if (pn) return pn;
    } catch { /* abaikan, pakai apa adanya */ }
  }
  return raw;
}

async function onMsg(id, m) {
  if (m.type !== 'notify') return;
  const sock = sessions.get(id)?.sock;
  for (const msg of m.messages || []) {
    if (!msg?.message || msg.key?.fromMe) continue;
    const raw = msg.key?.remoteJid || '';
    if (raw.endsWith('@g.us') || raw === 'status@broadcast') continue; // 1-to-1 saja
    const text = msg.message.conversation || msg.message.extendedTextMessage?.text;
    if (!text) continue;
    const jid = await resolvePn(sock, msg.key); // @lid → nomor asli
    postEvent(id, 'message', {
      remote_jid: jid,
      body: text,
      wa_msg_id: msg.key?.id || null,
      ts: Number(msg.messageTimestamp) || 0,
    });
  }
}

// --- Outbound queue ber-throttle (anti-ban): jeda acak + simulasi mengetik + cap jam/hari ---
function enqueue(id, to, text, media) {
  const s = sessions.get(id);
  if (!s) return { ok: false, error: 'unknown_session' };
  s.q = s.q || [];
  s.q.push({ to, text, media: media || null });
  if (!s.pumping) pump(id).catch((e) => app.log.error(`pump ${id}: ${e.message}`));
  return { ok: true, position: s.q.length };
}

// Bangun konten pesan Baileys dari item antrian (teks atau media via URL).
function buildContent(msg) {
  const m = msg.media;
  if (!m || !m.url) return { text: String(msg.text ?? '') };
  const caption = String(msg.text || m.caption || '');
  if (m.type === 'image') return { image: { url: m.url }, caption };
  if (m.type === 'video') return { video: { url: m.url }, caption };
  return { document: { url: m.url }, fileName: m.filename || (m.url.split('/').pop() || 'file'), mimetype: m.mimetype || 'application/octet-stream', caption };
}

async function pump(id) {
  const s = sessions.get(id);
  if (!s || s.pumping) return;
  s.pumping = true;
  try {
    const fake = DRY || s.sim;
    while (s.q && s.q.length) {
      if (!fake && s.status !== 'connected') break; // tunggu reconnect; pump dipicu lagi saat 'open'
      rollWindows(s);
      const cfg = s.cfg || cfgFor('normal'); // baca tiap iterasi → perubahan level berlaku live
      if ((s.hCnt || 0) >= cfg.hourCap || (s.dCnt || 0) >= cfg.dayCap) {
        const reason = (s.hCnt || 0) >= cfg.hourCap ? 'hourly' : 'daily';
        postEvent(id, 'throttled', { reason, retry_in_s: 60 });
        await sleep(60000);
        continue;
      }
      const msg = s.q[0];
      // `to` boleh berupa JID lengkap ("...@lid"/"...@s.whatsapp.net") atau angka biasa.
      const toRaw = String(msg.to);
      let jid = toRaw.includes('@') ? toRaw : (toRaw.replace(/\D/g, '') + '@s.whatsapp.net');
      // Kalau masih @lid, terjemahkan ke nomor asli dulu supaya benar-benar terkirim.
      if (!fake && jid.endsWith('@lid')) {
        try { const pn = await s.sock?.signalRepository?.lidMapping?.getPNForLID?.(jid); if (pn) jid = pn; } catch { /* pakai @lid apa adanya */ }
      }
      const typ = Math.min(cfg.typMax, Math.max(cfg.typMin, (msg.text || '').length * 60));
      let ok = false, waid = null, err = null;
      try {
        if (fake) {
          await sleep(Math.min(typ, 400));
          ok = true; waid = (s.sim ? 'sim-' : 'dry-') + Date.now();
        } else {
          await s.sock.sendPresenceUpdate('composing', jid).catch(() => {});
          await sleep(typ);
          await s.sock.sendPresenceUpdate('paused', jid).catch(() => {});
          const r = await s.sock.sendMessage(jid, buildContent(msg));
          ok = true; waid = r?.key?.id || null;
        }
        s.hCnt = (s.hCnt || 0) + 1;
        s.dCnt = (s.dCnt || 0) + 1;
      } catch (e) { err = e.message; }
      s.q.shift();
      postEvent(id, 'sent', { remote_jid: jid, body: msg.text, wa_msg_id: waid, status: ok ? 'sent' : 'failed', error: err });
      if (s.q.length) await sleep(rand(cfg.minGap, cfg.maxGap)); // jeda acak antar pesan
    }
  } finally {
    s.pumping = false;
  }
}

// --- HTTP ---
const app = Fastify({ logger: { level: 'info' } });

// Guard: semua endpoint butuh header X-WA-Secret (constant-time).
app.addHook('onRequest', async (req, reply) => {
  const got = Buffer.from(String(req.headers['x-wa-secret'] || ''));
  const want = Buffer.from(SECRET);
  if (got.length !== want.length || !timingSafeEqual(got, want)) {
    reply.code(401).send({ error: 'unauthorized' });
  }
});

app.get('/health', async () => ({ ok: true, sessions: sessions.size }));

app.get('/sessions', async () => ({
  sessions: [...sessions.entries()].map(([id, s]) => ({ sessionId: id, status: s.status, hasQr: !!s.qr })),
}));

app.post('/sessions/start', async (req, reply) => {
  const { session_id, level, sim } = req.body || {};
  if (!session_id) return reply.code(400).send({ error: 'session_id required' });
  startSession(session_id, level, !!sim).catch((e) => app.log.error(`start ${session_id}: ${e.message}`));
  return reply.code(202).send({ sessionId: session_id, status: sim ? 'connected' : 'connecting', level: level || 'normal', sim: !!sim });
});

// Ubah level anti-ban LIVE (tanpa reconnect).
app.post('/sessions/:id/level', async (req, reply) => {
  const s = sessions.get(req.params.id);
  if (!s) return reply.code(404).send({ error: 'unknown_session' });
  const level = (req.body || {}).level;
  s.level = level || 'normal';
  s.cfg = cfgFor(s.level);
  return { ok: true, level: s.level, cfg: s.cfg };
});

app.get('/sessions/:id/qr', async (req) => {
  const s = sessions.get(req.params.id);
  return { status: s?.status || 'unknown', qr: s?.qr || null };
});

app.get('/sessions/:id/status', async (req) => {
  const s = sessions.get(req.params.id);
  if (!s) return { sessionId: req.params.id, status: 'unknown' };
  return { sessionId: req.params.id, status: s.status, phone: s.phone || null, ...capStats(s) };
});

// Kirim pesan → masuk antrian ber-throttle (BUKAN langsung). Hasil dilaporkan via event 'sent'.
app.post('/sessions/:id/send', async (req, reply) => {
  const s = sessions.get(req.params.id);
  if (!s) return reply.code(404).send({ error: 'unknown_session' });
  const { to, text } = req.body || {};
  if (!String(to || '').replace(/\D/g, '')) return reply.code(400).send({ error: 'bad_to' });
  const r = enqueue(req.params.id, to, String(text ?? ''));
  return reply.code(202).send({ queued: true, position: r.position, ...capStats(s) });
});

// Kirim MEDIA (gambar/dokumen/video) via URL → masuk antrian throttle (sama seperti teks).
app.post('/sessions/:id/send-media', async (req, reply) => {
  const s = sessions.get(req.params.id);
  if (!s) return reply.code(404).send({ error: 'unknown_session' });
  const { to, media_url, type, caption } = req.body || {};
  if (!String(to || '').replace(/\D/g, '')) return reply.code(400).send({ error: 'bad_to' });
  if (!media_url) return reply.code(400).send({ error: 'media_url required' });
  const media = { url: String(media_url), type: ['image', 'video', 'document'].includes(type) ? type : 'document' };
  const r = enqueue(req.params.id, to, String(caption ?? ''), media);
  return reply.code(202).send({ queued: true, position: r.position, ...capStats(s) });
});

// Suntik pesan MASUK simulasi (hanya sesi sim / mode DRY) — uji E2E tanpa nomor nyata.
app.post('/sessions/:id/sim/incoming', async (req, reply) => {
  const s = sessions.get(req.params.id);
  if (!s) return reply.code(404).send({ error: 'unknown_session' });
  if (!(DRY || s.sim)) return reply.code(400).send({ error: 'not_sim', message: 'nomor ini bukan mode simulasi' });
  const { from, text } = req.body || {};
  const digits = String(from || '').replace(/\D/g, '') || '628000000001';
  postEvent(req.params.id, 'message', {
    remote_jid: digits + '@s.whatsapp.net',
    body: String(text ?? ''),
    wa_msg_id: 'sim-in-' + Date.now(),
    ts: Math.floor(Date.now() / 1000),
  });
  return { ok: true };
});

app.post('/sessions/:id/logout', async (req) => {
  const id = req.params.id;
  const s = sessions.get(id);
  if (s) { try { await s.sock.logout(); } catch { /* ignore */ } }
  await wipeAuth(id);
  sessions.delete(id);
  return { ok: true };
});

app.delete('/sessions/:id', async (req) => {
  const id = req.params.id;
  const s = sessions.get(id);
  if (s) { try { s.sock.end(); } catch { /* ignore */ } sessions.delete(id); }
  if (req.query?.purge === '1') await wipeAuth(id);
  return { ok: true };
});

app.listen({ port: PORT, host: '127.0.0.1' })
  .then(() => app.log.info(`wa-sidecar siap di http://127.0.0.1:${PORT} (RUST_URL=${RUST_URL})`))
  .catch((e) => { console.error(e); process.exit(1); });
