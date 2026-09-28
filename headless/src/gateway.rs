use crate::{
    CLOUDOS_ROOT,
    audit::{self, AuditRecord},
    auth, devices, execute_named, now_secs, output_hash,
};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct GatewayState {
    pub secret: String,
    pub server: String,
    pub audit_path: PathBuf,
    pub devices_path: PathBuf,
    pub replay: Arc<Mutex<HashMap<String, u64>>>,
}

#[derive(Deserialize)]
struct ExecuteRequest {
    protocol: String,
    request_id: String,
    timestamp: u64,
    nonce: String,
    principal: String,
    operation: String,
    root: Option<String>,
}

#[derive(Serialize)]
struct ExecuteResponse {
    request_id: String,
    success: bool,
    exit_code: Option<i32>,
    output: String,
    audit_hash: String,
}

pub async fn serve(bind: &str, state: GatewayState) -> Result<(), String> {
    let addr: SocketAddr = bind.parse().map_err(|_| "invalid gateway bind")?;
    if !matches!(addr.ip(), IpAddr::V4(value) if value.is_loopback())
        && !matches!(addr.ip(), IpAddr::V6(value) if value.is_loopback())
    {
        return Err(
            "gateway must bind to loopback; terminate TLS/authenticated transport in front of it"
                .into(),
        );
    }

    let app = Router::new()
        .route("/v1/health", get(health))
        .route("/v1/execute", post(execute))
        .route("/v1/audit", get(audit_log))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| e.to_string())?;
    axum::serve(listener, app).await.map_err(|e| e.to_string())
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "ok": true,
        "protocol": "KMJ-COMMANDER/1",
        "auth": "short-lived-hmac",
        "policy": "deny-by-default"
    }))
}

fn authenticate(
    headers: &HeaderMap,
    state: &GatewayState,
    scope: &str,
) -> Result<auth::TokenClaims, (StatusCode, String)> {
    let header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok());
    let claims = auth::verify_bearer(header, &state.secret, now_secs(), &state.server)
        .map_err(|error| (StatusCode::UNAUTHORIZED, error))?;
    if !claims.scopes.iter().any(|candidate| candidate == scope) {
        return Err((StatusCode::FORBIDDEN, "scope denied".into()));
    }
    match devices::is_active(&state.devices_path, &claims.device) {
        Ok(true) => Ok(claims),
        Ok(false) => Err((StatusCode::FORBIDDEN, "device revoked or not paired".into())),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "device registry unavailable".into())),
    }
}

fn scope_for(operation: &str) -> Option<&'static str> {
    match operation {
        "commander.probe" => Some("commander:read"),
        "cloudos.inspect" | "cloudos.git_status" | "cloudos.diff_check" => Some("cloudos:read"),
        "cloudos.py_compile" | "cloudos.provider_tests" | "cloudos.full_tests" => {
            Some("cloudos:test")
        }
        _ => None,
    }
}

async fn execute(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    Json(request): Json<ExecuteRequest>,
) -> impl IntoResponse {
    let Some(scope) = scope_for(&request.operation) else {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "unknown operation denied"})),
        )
            .into_response();
    };
    let claims = match authenticate(&headers, &state, scope) {
        Ok(claims) => claims,
        Err((status, error)) => {
            return (status, Json(serde_json::json!({"error": error}))).into_response();
        }
    };

    if request.protocol != "KMJ-COMMANDER/1"
        || Uuid::parse_str(&request.request_id).is_err()
        || request.nonce.len() < 16
        || request.principal != claims.sub
        || request.timestamp.abs_diff(now_secs()) > 60
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "invalid request envelope"})),
        )
            .into_response();
    }

    {
        let mut replay = state.replay.lock().unwrap();
        let now = now_secs();
        replay.retain(|_, expires| *expires >= now);
        let key = format!("{}:{}:{}", claims.jti, request.request_id, request.nonce);
        if replay.insert(key, claims.exp).is_some() {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({"error": "replayed request"})),
            )
                .into_response();
        }
    }

    let root = request.root.as_deref().unwrap_or(CLOUDOS_ROOT);
    let outcome = execute_named(root, &request.operation);
    let (success, exit_code, output) = match outcome {
        Ok(value) => (value.success, value.exit_code, value.output),
        Err(error) => (false, None, error),
    };
    let digest = output_hash(&output);
    let event_id = Uuid::new_v4().to_string();
    let record = AuditRecord {
        event_id,
        request_id: request.request_id.clone(),
        timestamp: now_secs(),
        principal: claims.sub,
        server: claims.server,
        device: claims.device,
        operation: request.operation,
        outcome: if success {
            "success".into()
        } else {
            "failure".into()
        },
        exit_code,
        output_sha256: digest,
        previous_hash: String::new(),
        record_hash: String::new(),
    };
    let audit_hash = match audit::append(&state.audit_path, record) {
        Ok(hash) => hash,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("audit failure: {error}")})),
            )
                .into_response();
        }
    };

    (
        if success {
            StatusCode::OK
        } else {
            StatusCode::UNPROCESSABLE_ENTITY
        },
        Json(
            serde_json::to_value(ExecuteResponse {
                request_id: request.request_id,
                success,
                exit_code,
                output,
                audit_hash,
            })
            .unwrap(),
        ),
    )
        .into_response()
}

async fn audit_log(State(state): State<GatewayState>, headers: HeaderMap) -> impl IntoResponse {
    if let Err((status, error)) = authenticate(&headers, &state, "audit:read") {
        return (status, Json(serde_json::json!({"error": error}))).into_response();
    }
    match audit::read_bounded(&state.audit_path, 100) {
        Ok(lines) => (StatusCode::OK, Json(serde_json::json!({"records": lines}))).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": error})),
        )
            .into_response(),
    }
}
