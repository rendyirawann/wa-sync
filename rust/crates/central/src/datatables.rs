use std::collections::HashMap;

use serde_json::{json, Value};

/// Parameter standar request DataTables server-side (subset yang kita pakai).
pub struct DtParams {
    pub draw: i64,
    pub start: i64,
    pub length: i64,
    pub search: String,
}

/// Parse dari query map (axum `Query<HashMap<String,String>>`).
/// Kunci memakai notasi DataTables: `draw`, `start`, `length`, `search[value]`.
pub fn parse(q: &HashMap<String, String>) -> DtParams {
    DtParams {
        draw: q.get("draw").and_then(|s| s.parse().ok()).unwrap_or(0),
        start: q.get("start").and_then(|s| s.parse().ok()).unwrap_or(0),
        length: q.get("length").and_then(|s| s.parse().ok()).unwrap_or(10),
        search: q.get("search[value]").cloned().unwrap_or_default(),
    }
}

impl DtParams {
    /// LIMIT untuk SQL (length -1 = semua → pakai angka besar).
    pub fn limit(&self) -> i64 {
        if self.length < 0 { 1_000_000 } else { self.length }
    }
}

/// Bungkus respons JSON DataTables.
pub fn response(draw: i64, records_total: i64, records_filtered: i64, data: Vec<Value>) -> Value {
    json!({
        "draw": draw,
        "recordsTotal": records_total,
        "recordsFiltered": records_filtered,
        "data": data,
    })
}
