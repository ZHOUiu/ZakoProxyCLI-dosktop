//! JNI host for the Android build.
//!
//! The Android app runs the relay inside its own process: `Runtime.exec` cannot start a
//! binary from app storage on Android 10+, so instead of shipping an executable the whole
//! server (router + static console + `/v1` relay) is linked as `libzakoai.so` and driven
//! from a native thread that this module creates.

use std::sync::OnceLock;

use jni::objects::{JClass, JString};
use jni::JNIEnv;
use tokio::net::TcpListener;

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

/// Builds the app state and serves it until the runtime stops.
fn serve(data: String, web: String, bind: String) {
    let runtime = RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime")
    });
    runtime.block_on(async move {
        let state = match crate::new_app(std::path::Path::new(&data), web.into()) {
            Ok(state) => state,
            Err(error) => {
                eprintln!("state init failed: {error}");
                return;
            }
        };
        let listener = match TcpListener::bind(&bind).await {
            Ok(listener) => listener,
            Err(error) => {
                eprintln!("bind {bind} failed: {error}");
                return;
            }
        };
        println!("ZakoAI embedded server listening on {bind}");
        if let Err(error) = axum::serve(listener, crate::app(state)).await {
            eprintln!("embedded server stopped: {error}");
        }
    });
}

fn java_string(env: &mut JNIEnv, value: &JString) -> String {
    env.get_string(value)
        .map(|text| text.into())
        .unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_zakoai_proxy_ProxyProcess_nativeServe(
    mut env: JNIEnv,
    _class: JClass,
    data_dir: JString,
    web_dir: JString,
    bind: JString,
) {
    let data = java_string(&mut env, &data_dir);
    let web = java_string(&mut env, &web_dir);
    let bind_address = java_string(&mut env, &bind);
    // The JVM thread that enters here is attached to the runtime; keep serving on it.
    serve(data, web, bind_address);
}
