//! Render body broadcast: variabel `{{key}}` + spintax `{a|b|c}` (boleh nested).
//! Pemilihan spintax DETERMINISTIK per-target (seed = sha256 dari identitas target),
//! jadi reproducible saat retry tapi tetap variatif antar-kontak. Tanpa dependency baru
//! (pakai `sha2` yang sudah ada — tanpa `rand`/`regex`).

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

/// Penghasil indeks pseudo-acak deterministik dari seed (hash bergulir per pemanggilan).
struct Picker {
    seed: [u8; 32],
    counter: u32,
}

impl Picker {
    fn new(parts: &[&str]) -> Self {
        let mut h = Sha256::new();
        for p in parts {
            h.update(p.as_bytes());
            h.update([0u8]);
        }
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&h.finalize());
        Self { seed, counter: 0 }
    }
    fn pick(&mut self, n: usize) -> usize {
        if n <= 1 {
            return 0;
        }
        let mut h = Sha256::new();
        h.update(self.seed);
        h.update(self.counter.to_le_bytes());
        self.counter += 1;
        let d = h.finalize();
        (u32::from_le_bytes([d[0], d[1], d[2], d[3]]) as usize) % n
    }
}

/// Pisah '|' hanya di kedalaman brace 0 (alternatif spintax level atas).
fn split_top_alts(s: &str) -> Vec<&str> {
    let mut depth: i32 = 0;
    let mut start = 0usize;
    let mut out = Vec::new();
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            '|' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

/// Resolusi spintax rekursif (brace seimbang). Brace tak seimbang dibiarkan apa adanya.
fn resolve_spintax(s: &str, p: &mut Picker) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'{' {
            // Placeholder variabel `{{...}}` bukan spintax — salin apa adanya,
            // biar diproses tahap variabel. (Mencegah `{{nama}}` ditelan spintax.)
            if i + 1 < b.len() && b[i + 1] == b'{' {
                if let Some(rel) = s[i + 2..].find("}}") {
                    let end = i + 2 + rel + 2;
                    out.push_str(&s[i..end]);
                    i = end;
                    continue;
                }
            }
            let mut depth: i32 = 0;
            let mut j = i;
            let mut close = None;
            while j < b.len() {
                match b[j] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            close = Some(j);
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            if let Some(end) = close {
                let alts = split_top_alts(&s[i + 1..end]);
                let chosen = alts[p.pick(alts.len())];
                out.push_str(&resolve_spintax(chosen, p));
                i = end + 1;
                continue;
            }
        }
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Ganti `{{key}}` dengan nilai dari `vars` (string apa adanya, lainnya di-stringify, hilang = kosong).
fn replace_vars(s: &str, vars: &Map<String, Value>) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < b.len() {
        if i + 1 < b.len() && b[i] == b'{' && b[i + 1] == b'{' {
            if let Some(rel) = s[i + 2..].find("}}") {
                let key = s[i + 2..i + 2 + rel].trim();
                let val = match vars.get(key) {
                    Some(Value::String(v)) => v.clone(),
                    Some(Value::Null) | None => String::new(),
                    Some(other) => other.to_string(),
                };
                out.push_str(&val);
                i = i + 2 + rel + 2;
                continue;
            }
        }
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Render lengkap: spintax dulu (struktur), lalu variabel (isi).
/// `seed_parts` = identitas target unik, mis. `[campaign_id, phone]`.
pub fn render(body: &str, vars: &Map<String, Value>, seed_parts: &[&str]) -> String {
    let mut p = Picker::new(seed_parts);
    let spun = resolve_spintax(body, &mut p);
    replace_vars(&spun, vars)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn vars(j: Value) -> Map<String, Value> {
        j.as_object().unwrap().clone()
    }

    #[test]
    fn var_replace() {
        let v = vars(json!({"nama": "Budi"}));
        assert_eq!(render("Halo {{nama}}!", &v, &["c", "62811"]), "Halo Budi!");
        // var hilang → kosong
        assert_eq!(render("Hai {{kota}}", &v, &["c", "62811"]), "Hai ");
    }

    #[test]
    fn spintax_picks_one() {
        let v = Map::new();
        let r = render("{Halo|Hai|Hi} dunia", &v, &["c", "62811"]);
        assert!(["Halo dunia", "Hai dunia", "Hi dunia"].contains(&r.as_str()), "got: {r}");
    }

    #[test]
    fn spintax_nested() {
        let v = Map::new();
        let r = render("{Halo {pak|bu}|Hai}", &v, &["c", "62811"]);
        assert!(["Halo pak", "Halo bu", "Hai"].contains(&r.as_str()), "got: {r}");
    }

    #[test]
    fn deterministic_same_seed() {
        let v = vars(json!({"nama": "Sari"}));
        let body = "{Halo|Hai|Hi} {{nama}}, {promo A|promo B|promo C}";
        let a = render(body, &v, &["camp1", "62812345"]);
        let b = render(body, &v, &["camp1", "62812345"]);
        assert_eq!(a, b, "seed sama harus identik");
    }

    #[test]
    fn different_seed_varies() {
        // Antar-kontak berbeda seed → distribusi bervariasi (cek minimal tak panik & valid).
        let v = Map::new();
        let body = "{A|B|C|D|E}";
        let mut seen = std::collections::HashSet::new();
        for i in 0..50 {
            let phone = format!("6281{i:07}");
            seen.insert(render(body, &v, &["camp1", &phone]));
        }
        assert!(seen.len() > 1, "harus ada variasi antar-kontak");
    }
}
