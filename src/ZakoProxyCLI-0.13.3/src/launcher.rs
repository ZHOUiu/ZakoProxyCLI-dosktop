use std::ffi::c_void;

const STACK: usize = 64 * 1024 * 1024;

#[cfg(all(target_os = "windows", target_env = "msvc"))]
mod imp {
    use super::STACK;
    use std::ffi::c_void;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateThread(
            attributes: *mut c_void,
            stack_size: usize,
            start: extern "system" fn(*mut c_void) -> u32,
            parameter: *mut c_void,
            flags: u32,
            thread_id: *mut u32,
        ) -> *mut c_void;
    }

    extern "system" fn trampoline(_: *mut c_void) -> u32 {
        super::run();
        0
    }

    pub fn start() {
        let mut id: u32 = 0;
        let handle = unsafe {
            CreateThread(
                std::ptr::null_mut(),
                STACK,
                trampoline,
                std::ptr::null_mut(),
                0,
                &mut id,
            )
        };
        if handle.is_null() {
            // Fall back to the main thread stack if the big stack could not be allocated.
            super::run();
        }
    }
}

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
mod imp {
    use super::STACK;

    pub fn start() {
        let result = std::thread::Builder::new()
            .name("zakoai-server".into())
            .stack_size(STACK)
            .spawn(super::run);
        if result.is_err() {
            super::run();
        } else {
            // Keep the launcher alive while the server thread runs.
            loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
            }
        }
    }
}

/// Entry point used by the Windows launcher shim: builds the app and serves the console.
pub fn run() {
    let base = base_dir();
    let _ = std::env::set_current_dir(&base);
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "zakoai_proxy=info".into()),
        )
        .init();
    let data = resolve_dir(&base, &std::env::var("DATA_DIR").unwrap_or_else(|_| "data".into()));
    let web = resolve_dir(&base, &std::env::var("WEB_DIR").unwrap_or_else(|_| "web".into()));
    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:1990".into());
    let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::error!(error = %error, "runtime");
            return;
        }
    };
    runtime.block_on(async move {
        let state = match crate::new_app(std::path::Path::new(&data), web.into()) {
            Ok(state) => state,
            Err(error) => {
                tracing::error!(error = %error, "state");
                return;
            }
        };
        let listener = match tokio::net::TcpListener::bind(&bind).await {
            Ok(listener) => listener,
            Err(error) => {
                tracing::error!(error = %error, address = %bind, "bind failed");
                return;
            }
        };
        tracing::info!(address = %bind, "ZakoAI is listening");
        let _ = axum::serve(listener, crate::app(state)).await;
    });
}

fn base_dir() -> std::path::PathBuf {
    if let Ok(explicit) = std::env::var("ZAKO_HOME") {
        return std::path::PathBuf::from(explicit);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if dir.join("web").is_dir() || dir.join("data").is_dir() {
                return dir.to_path_buf();
            }
        }
    }
    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
}

fn resolve_dir(base: &std::path::Path, value: &str) -> String {
    let path = std::path::Path::new(value);
    if path.is_absolute() {
        value.to_owned()
    } else {
        base.join(path).to_string_lossy().into_owned()
    }
}

/// Exported for the Windows launcher: runs the console server on a 64 MB stack thread.
#[unsafe(no_mangle)]
pub extern "C" fn zako_launch(_instance: *mut c_void) {
    imp::start();
}
