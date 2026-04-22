// aura-app/src/lib.rs

pub mod hot_swap;

use tauri::{Manager, State};
use std::sync::Arc;
use url::Url;
use hot_swap::{HotSwapManager, SwapError};

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum AuraError {
    #[error("Invalid URL")]
    InvalidUrl(String),
    #[error("Engine error: {0}")]
    EngineError(String),
}

impl From<SwapError> for AuraError {
    fn from(err: SwapError) -> Self {
        AuraError::EngineError(err.to_string())
    }
}

#[tauri::command]
async fn navigate(
    url: String,
    manager: State<'_, Arc<HotSwapManager>>,
) -> Result<(), AuraError> {
    // 1. Validate & sanitise URL
    let parsed = Url::parse(&url).map_err(|_| AuraError::InvalidUrl(url.clone()))?;

    // 2. Pass through network interceptor (adblock)
    // For now, we'll assume a source_url and resource_type
    let source_url = parsed.clone();
    let filtered = aura_net::intercept(&parsed, &source_url, "main_frame").await;

    match filtered {
        aura_net::InterceptDecision::Allow(target_url) | aura_net::InterceptDecision::Redirect(target_url) => {
            // 3. Forward to hot-loaded engine
            manager.navigate(target_url.as_str()).await?;
            Ok(())
        }
        aura_net::InterceptDecision::Block { reason } => {
            Err(AuraError::EngineError(format!("Blocked: {}", reason)))
        }
    }
}

#[tokio::main]
pub async fn run() {
    let manager = Arc::new(HotSwapManager::new());
    
    // Attempt to load the initial engine
    let engine_path = if cfg!(windows) {
        "./engines/aura_engine.dll"
    } else if cfg!(target_os = "macos") {
        "./engines/libaura_engine.dylib"
    } else {
        "./engines/libaura_engine.so"
    };

    let _ = manager.load_initial_engine(engine_path.into()).await;

    tauri::Builder::default()
        .manage(manager)
        .setup(|app| {
            let win = app.get_webview_window("main").unwrap();
            
            // Borderless & Transparent
            let _ = win.set_decorations(false);
            let _ = win.set_shadow(false); 
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![navigate])
        .run(tauri::generate_context!())
        .expect("Aura failed to launch");
}
