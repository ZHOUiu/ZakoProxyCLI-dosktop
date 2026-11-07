#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = base_dir();
    std::env::set_current_dir(&base)?;
    if base.join(".env").exists() {
        dotenvy::from_path(base.join(".env"))?;
    }
    if std::env::var("ZAKO_NO_CONSOLE").is_err() && std::env::var("RUST_LOG").is_err() {
        // Console builds run under pythonw: keep logs in a file instead of a console.
        let log_dir = base.join("logs");
        let _ = std::fs::create_dir_all(&log_dir);
        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_dir.join("zakoai.log"))
        {
            tracing_subscriber::fmt()
                .with_writer(std::sync::Mutex::new(file))
                .with_ansi(false)
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "zakoai_proxy=info".into()),
                )
                .init();
        }
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "zakoai_proxy=info".into()),
            )
            .init();
    }
    let data = std::env::var("DATA_DIR").unwrap_or_else(|_| "data".into());
    let web = std::env::var("WEB_DIR").unwrap_or_else(|_| "web".into());
    let web = resolve_dir(&base, &web);
    let data = resolve_dir(&base, &data);
    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:1990".into());
    let state = zakoai_proxy::new_app(std::path::Path::new(&data), web.into())?;
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(address=%bind,"ZakoAI is listening");
    axum::serve(listener, zakoai_proxy::app(state))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

/// Directory holding data/, web/ and .env: next to the executable when packaged.
fn base_dir() -> std::path::PathBuf {    if let Ok(explicit) = std::env::var("ZAKO_HOME") {
        return std::path::PathBuf::from(explicit);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            // Only prefer the executable directory when it already carries the bundle.
            if dir.join("web").is_dir() || dir.join("data").is_dir() {
                return dir.to_path_buf();
            }
        }
    }
    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
}

/// Resolves a configured directory against the bundle root when it is relative.
fn resolve_dir(base: &std::path::Path, value: &str) -> String {
    let path = std::path::Path::new(value);
    if path.is_absolute() {
        value.to_owned()
    } else {
        base.join(path).to_string_lossy().into_owned()
    }
}
