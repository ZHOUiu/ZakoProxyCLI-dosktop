use axum::{
    Router,
    body::Body,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use store::Store;

pub mod api;
pub mod launcher;
#[cfg(target_os = "android")]
pub mod embedded;
pub mod network;
pub mod playground;
pub mod profiles;
pub mod provider_keys;
pub mod providers;
pub mod presets;
pub mod proxy;
pub mod store;
pub mod update;

#[derive(Default, Clone, serde::Serialize)]
pub struct Runtime {
    pub active: u32,
    pub failures: u32,
    pub cooldown_until: i64,
    #[serde(skip)]
    pub window: i64,
    #[serde(skip)]
    pub requests: u32,
}
pub struct App {
    pub store: Store,
    pub turnstile_site_key: std::sync::RwLock<String>,
    pub turnstile_secret_key: std::sync::RwLock<String>,
    pub turnstile_env_file: std::path::PathBuf,
    pub client: reqwest::Client,
    pub proxy_clients: Mutex<HashMap<String, (String, reqwest::Client)>>,
    pub runtime: Mutex<HashMap<String, Runtime>>,
    pub login_attempts: Mutex<HashMap<String, (i64, u32)>>,
    pub(crate) provider_preflights: Mutex<HashMap<String, (i64, providers::Provider)>>,
    pub web_dir: std::path::PathBuf,
}

pub type Shared = Arc<App>;
pub type ApiResult<T> = Result<axum::Json<T>, ApiError>;

#[derive(Debug)]
pub struct ApiError(pub StatusCode, pub String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, axum::Json(json!({"error":self.1}))).into_response()
    }
}

impl From<String> for ApiError {
    fn from(value: String) -> Self {
        tracing::error!(error=%value, "Storage or validation error");
        Self(StatusCode::BAD_REQUEST, value)
    }
}

impl From<rusqlite::Error> for ApiError {
    fn from(value: rusqlite::Error) -> Self {
        tracing::error!(error=%value, "Database error");
        Self(StatusCode::INTERNAL_SERVER_ERROR, "Storage failure".into())
    }
}

pub fn unauthorized() -> ApiError {
    ApiError(StatusCode::UNAUTHORIZED, "Please sign in".into())
}

pub fn bad(msg: impl Into<String>) -> ApiError {
    ApiError(StatusCode::BAD_REQUEST, msg.into())
}

/// Persists Turnstile keys into the project .env next to the data directory.
pub fn save_turnstile_keys(
    app: &App,
    site_key: &str,
    secret_key: &str,
) -> Result<(), String> {
    let replacements = [
        ("TURNSTILE_SITE_KEY", site_key),
        ("TURNSTILE_SECRET_KEY", secret_key),
    ];
    let existing = std::fs::read_to_string(&app.turnstile_env_file).unwrap_or_default();
    let mut lines: Vec<String> = existing
        .lines()
        .map(|line| {
            let key = line.split('=').next().unwrap_or_default().trim();
            if replacements.iter().any(|(name, _)| *name == key) {
                String::new()
            } else {
                line.to_owned()
            }
        })
        .filter(|line| !line.is_empty())
        .collect();
    for (name, value) in replacements {
        lines.push(format!("{name}={value}"));
    }
    let mut contents = lines.join("\n");
    contents.push('\n');
    std::fs::write(&app.turnstile_env_file, contents)
        .map_err(|error| format!("Could not update project .env: {error}"))?;
    if let Ok(mut slot) = app.turnstile_site_key.write() {
        *slot = site_key.to_owned();
    }
    if let Ok(mut slot) = app.turnstile_secret_key.write() {
        *slot = secret_key.to_owned();
    }
    Ok(())
}

impl App {
    pub fn save_turnstile_keys(&self, site_key: &str, secret_key: &str) -> Result<(), String> {
        save_turnstile_keys(self, site_key, secret_key)
    }
}
impl App {
    pub fn turnstile_site_key(&self) -> String {
        self.turnstile_site_key.read().map(|v| v.clone()).unwrap_or_default()
    }
    pub fn turnstile_secret_key(&self) -> String {
        self.turnstile_secret_key.read().map(|v| v.clone()).unwrap_or_default()
    }
    pub fn turnstile_env_path(&self) -> &std::path::Path {
        &self.turnstile_env_file
    }
}
pub fn app(state: Shared) -> Router {
    Router::new()
        .route(
            "/health",
            get(|| async { axum::Json(json!({"status":"ok"})) }),
        )
        .route("/api/bootstrap", get(api::bootstrap))
        .route("/api/setup", post(api::setup))
        .route("/api/login", post(api::login))
        .route("/api/logout", post(api::logout))
        .route("/api/groups", get(api::groups).post(api::save_group))
        .route(
            "/api/groups/{id}",
            put(api::save_group_id).delete(api::delete_group),
        )
        .route("/api/accounts", get(api::accounts).post(api::save_account))
        .route("/api/accounts/proxy-test", post(api::test_proxy))
        .route(
            "/api/accounts/{id}/provider-binding",
            get(provider_keys::refresh_binding),
        )
        .route(
            "/api/accounts/{id}",
            put(api::save_account_id).delete(api::delete_account),
        )
        .route("/api/keys", get(api::keys).post(api::create_key))
        .route(
            "/api/keys/{id}",
            put(api::update_key).delete(api::delete_key),
        )
        .route("/api/keys/{id}/reveal", post(api::reveal_key))
        .route(
            "/api/providers",
            get(providers::list).post(providers::create),
        )
        .route("/api/providers/test", post(providers::test))
        .route(
            "/api/providers/auth-requirements",
            post(providers::auth_requirements_input),
        )
        .route(
            "/api/providers/{id}/auth-requirements",
            get(providers::auth_requirements),
        )
        .route(
            "/api/providers/{id}",
            put(providers::edit).delete(providers::delete),
        )
        .route("/api/providers/{id}/refresh", post(providers::refresh))
        .route(
            "/api/providers/{id}/keys",
            get(provider_keys::list).post(provider_keys::create),
        )
        .route("/api/providers/{id}/groups", get(provider_keys::groups))
        .route("/api/settings", get(api::settings).put(api::save_settings))
        .route("/api/settings/turnstile-test", post(api::test_turnstile))
        .route("/api/password", post(api::password))
        .route("/api/updates", get(update::status).post(update::start))
        .route("/api/updates/latest", get(update::latest))
        .route("/api/stats", get(api::stats))
        .route("/api/logs", get(api::logs).delete(api::delete_logs))
        .route("/api/logs/export", get(api::export_logs))
        .route("/api/profiles", get(api::profiles))
        .route("/api/models/discover", post(api::discover))
        .route("/api/presets", get(api::presets))
        .route("/api/accounts/parse-credential", post(api::parse_credential))
        .route("/api/playground/models", get(playground::models))
        .route("/api/playground/attachment", post(playground::attachment))
        .route("/api/playground/search", post(playground::search))
        .route("/api/playground", post(playground::chat))
        .route("/api/playground/chat", post(playground::chat))
        .route("/v1/models", get(proxy::models))
        .route("/v1/responses", post(proxy::responses))
        .route("/v1/responses/compact", post(proxy::compact))
        .route("/v1/chat/completions", post(proxy::chat_completions))
        .route("/v1/messages", post(proxy::messages))
        .layer(DefaultBodyLimit::max(16 * 1024 * 1024))
        .fallback(console)
        .with_state(state)
}

/// Console fallback: serves the built SPA and keeps the admin path gated.
async fn console(State(state): State<Shared>, headers: HeaderMap, uri: Uri) -> Response {
    let path = uri.path();
    if path.starts_with("/api/") || path.starts_with("/v1/") {
        return StatusCode::NOT_FOUND.into_response();
    }
    let entry = state
        .store
        .config()
        .map(|c| format!("/{}", c.settings.security_path))
        .unwrap_or_else(|_| "/".into());
    let asset = path.starts_with("/assets/") || path == "/favicon.svg";
    if path == "/" && entry != "/" && api::valid_access_host(&headers)
        && state.store.authenticated(api::cookie(&headers))
    {
        let Ok(location) = entry.parse() else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };
        let mut response = Response::new(Body::empty());
        *response.status_mut() = StatusCode::TEMPORARY_REDIRECT;
        response
            .headers_mut()
            .insert(axum::http::header::LOCATION, location);
        return response;
    }
    if entry != "/" && path != entry && !asset {
        let mut response = Response::new(Body::from(SHELL));
        response.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            "text/html; charset=utf-8".parse().unwrap(),
        );
        response
            .headers_mut()
            .insert(axum::http::header::CACHE_CONTROL, "no-store".parse().unwrap());
        return response;
    }
    let file = state.web_dir.join(if asset { path.trim_start_matches('/') } else { "index.html" });
    let mime = if !asset {
        // "/", the console entry and SPA deep links all fall back to index.html.
        "text/html; charset=utf-8"
    } else {
        match path.rsplit('.').next() {
            Some("js") | Some("mjs") => "text/javascript; charset=utf-8",
            Some("css") => "text/css; charset=utf-8",
            Some("html") => "text/html; charset=utf-8",
            Some("svg") => "image/svg+xml",
            Some("json") | Some("map") => "application/json",
            Some("woff2") => "font/woff2",
            Some("woff") => "font/woff",
            Some("png") => "image/png",
            Some("txt") => "text/plain; charset=utf-8",
            _ => "application/octet-stream",
        }
    };
    match tokio::fs::read(file).await {
        Ok(bytes) => {
            let mut response = Response::new(Body::from(bytes));
            let headers = response.headers_mut();
            headers.insert(axum::http::header::CONTENT_TYPE, mime.parse().unwrap());
            headers.insert(
                axum::http::header::X_CONTENT_TYPE_OPTIONS,
                "nosniff".parse().unwrap(),
            );
            headers.insert(
                axum::http::header::CACHE_CONTROL,
                if asset {
                    "public, max-age=86400"
                } else {
                    "no-store"
                }
                .parse()
                .unwrap(),
            );
            response
        }
        Err(_) => {
            // A missing asset must never be answered with the SPA shell: browsers reject
            // it (module MIME check) and the console silently stays blank.
            if asset {
                return StatusCode::NOT_FOUND.into_response();
            }
            let mut response = Response::new(Body::from(SHELL));
            response.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                "text/html; charset=utf-8".parse().unwrap(),
            );
            response
        }
    }
}

/// Static landing page used while the bundled console has not been built or extracted.
const SHELL: &str = r#"<!doctype html><html><head><meta charset="utf-8"><title>ZakoAI Proxy</title></head><body><p>ZakoAI Proxy is running. build=MK2</p></body></html>"#;

pub fn new_app(data_dir: &std::path::Path, web_dir: std::path::PathBuf) -> Result<Shared, String> {
    let turnstile_env_file = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join(".env");
    let env_values = std::fs::read_to_string(&turnstile_env_file)
        .ok()
        .and_then(|contents| {
            dotenvy::from_read_iter(contents.as_bytes())
                .collect::<std::result::Result<std::collections::HashMap<_, _>, _>>()
                .ok()
        })
        .unwrap_or_default();
    Ok(Arc::new(App {
        store: Store::open(data_dir)?,
        turnstile_site_key: std::sync::RwLock::new(
            env_values
                .get("TURNSTILE_SITE_KEY")
                .cloned()
                .or_else(|| std::env::var("TURNSTILE_SITE_KEY").ok())
                .unwrap_or_default(),
        ),
        turnstile_secret_key: std::sync::RwLock::new(
            env_values
                .get("TURNSTILE_SECRET_KEY")
                .cloned()
                .or_else(|| std::env::var("TURNSTILE_SECRET_KEY").ok())
                .unwrap_or_default(),
        ),
        turnstile_env_file,
        client: reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| e.to_string())?,
        runtime: Mutex::new(HashMap::new()),
        proxy_clients: Mutex::new(HashMap::new()),
        login_attempts: Mutex::new(HashMap::new()),
        provider_preflights: Mutex::new(HashMap::new()),
        web_dir,
    }))
}






