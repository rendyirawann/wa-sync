use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use chrono::NaiveDateTime;
use serde_json::{json, Value};
use sqlx::types::Uuid;

use crate::{activity, datatables, rbac::CurrentUser, view, AppState};

pub async fn index(user: CurrentUser, session: tower_sessions::Session, State(state): State<AppState>) -> Html<String> {
    let ctx = view::base_context(&state, &session, &user, "log-activity").await;
    match state.tera.render("backend/help/log_activity/index.html", &ctx) {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}

#[derive(sqlx::FromRow)]
struct LogRow {
    log_name: Option<String>,
    description: String,
    created_at: Option<NaiveDateTime>,
    properties: Option<Value>,
    causer_name: Option<String>,
}

const DT_SQL: &str = "SELECT a.log_name, a.description, a.created_at, a.properties, u.name AS causer_name \
 FROM activity_log a LEFT JOIN users u ON u.id = a.causer_id \
 WHERE ($1::uuid IS NULL OR a.causer_id = $1::uuid) \
   AND ($2 = '' OR a.log_name ILIKE '%'||$2||'%' OR a.description ILIKE '%'||$2||'%') \
 ORDER BY a.created_at DESC LIMIT $3 OFFSET $4";
const DT_COUNT_SQL: &str = "SELECT count(*) FROM activity_log a \
 WHERE ($1::uuid IS NULL OR a.causer_id = $1::uuid) \
   AND ($2 = '' OR a.log_name ILIKE '%'||$2||'%' OR a.description ILIKE '%'||$2||'%')";

pub async fn datatable(user: CurrentUser, State(state): State<AppState>, Query(q): Query<HashMap<String, String>>) -> Response {
    let p = datatables::parse(&q);
    // Superadmin → semua; lainnya → hanya miliknya.
    let filter: Option<Uuid> = if user.is_superadmin() { None } else { Some(user.id) };

    let total: i64 = sqlx::query_scalar(DT_COUNT_SQL).bind(filter).bind("").fetch_one(&state.pool).await.unwrap_or(0);
    let filtered: i64 = sqlx::query_scalar(DT_COUNT_SQL).bind(filter).bind(&p.search).fetch_one(&state.pool).await.unwrap_or(0);
    let rows: Vec<LogRow> = sqlx::query_as(DT_SQL).bind(filter).bind(&p.search).bind(p.limit()).bind(p.start).fetch_all(&state.pool).await.unwrap_or_default();

    let data: Vec<Value> = rows.iter().map(|r| {
        let props = r.properties.clone().unwrap_or(Value::Null);
        let (ip, os, device) = activity::agent_cols(&props);
        json!({
            "log_name": r.log_name.clone().unwrap_or_else(|| "-".into()),
            "causer_id": r.causer_name.clone().unwrap_or_else(|| "System".into()),
            "description": if r.description.is_empty() { "-".to_string() } else { r.description.clone() },
            "ip": ip,
            "os": os,
            "device": device,
            "created_at": r.created_at.map(activity::format_id_full).unwrap_or_else(|| "-".into()),
        })
    }).collect();

    Json(datatables::response(p.draw, total, filtered, data)).into_response()
}

pub async fn show(user: CurrentUser, session: tower_sessions::Session, State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    let row: Option<(Option<String>, String, Option<Value>, Option<Uuid>, Option<String>)> = sqlx::query_as(
        "SELECT a.log_name, a.description, a.properties, a.causer_id, u.name FROM activity_log a LEFT JOIN users u ON u.id = a.causer_id WHERE a.id = $1",
    ).bind(id).fetch_optional(&state.pool).await.ok().flatten();
    let Some((log_name, description, properties, causer_id, causer_name)) = row else {
        return Redirect::to("/admin/log-activity").into_response();
    };
    // Non-superadmin hanya boleh lihat log miliknya.
    if !user.is_superadmin() && causer_id != Some(user.id) {
        return Redirect::to("/admin/log-activity").into_response();
    }
    let props = properties.unwrap_or(Value::Null);
    let (ip, os_browser, device) = activity::agent_cols(&props);
    let pretty = |v: Option<&Value>| v.map(|x| serde_json::to_string_pretty(x).unwrap_or_default()).unwrap_or_default();
    let new_v = props.get("new");
    let old_v = props.get("old").map(|o| {
        // hilangkan password dari old
        let mut o = o.clone();
        if let Some(obj) = o.as_object_mut() { obj.remove("password"); }
        o
    });

    let mut ctx = view::base_context(&state, &session, &user, "log-activity").await;
    ctx.insert("log_name", &log_name.unwrap_or_default());
    ctx.insert("description", &description);
    ctx.insert("causer_name", &causer_name.unwrap_or_else(|| "System".into()));
    ctx.insert("ip", &ip);
    ctx.insert("os_browser", &os_browser);
    ctx.insert("device_html", &device);
    ctx.insert("has_new", &new_v.is_some());
    ctx.insert("json_new", &pretty(new_v));
    ctx.insert("has_old", &old_v.is_some());
    ctx.insert("json_old", &old_v.as_ref().map(|o| serde_json::to_string_pretty(o).unwrap_or_default()).unwrap_or_default());
    ctx.insert("json_agent", &pretty(props.get("agent")));
    ctx.insert("json_request", &pretty(props.get("request")));
    ctx.insert("json_all", &serde_json::to_string_pretty(&props).unwrap_or_default());
    match state.tera.render("backend/help/log_activity/show.html", &ctx) {
        Ok(html) => Html(html).into_response(),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")).into_response(),
    }
}
