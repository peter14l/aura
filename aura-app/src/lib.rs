// aura-app/src/lib.rs

pub mod hot_swap;

use aura_ui::MainUI;
use hot_swap::{HotSwapManager, SwapError};
use slint::ComponentHandle;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use url::Url;

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

pub struct AppState {
    pub hot_swap: Arc<HotSwapManager>,
    pub ui: MainUI,
}

#[tauri::command]
async fn navigate(url: String, state: State<'_, AppState>) -> Result<(), AuraError> {
    let parsed = Url::parse(&url).map_err(|_| AuraError::InvalidUrl(url.clone()))?;
    let source_url = parsed.clone();
    let filtered = aura_net::intercept(&parsed, &source_url, "main_frame").await;

    match filtered {
        aura_net::InterceptDecision::Allow(target_url)
        | aura_net::InterceptDecision::Redirect(target_url) => {
            state.hot_swap.navigate(target_url.as_str()).await?;
            Ok(())
        }
        aura_net::InterceptDecision::Block { reason } => {
            Err(AuraError::EngineError(format!("Blocked: {}", reason)))
        }
    }
}

#[tauri::command]
fn toggle_command_bar(state: State<'_, AppState>) {
    let current = state.ui.get_command_bar_visible();
    state.ui.set_command_bar_visible(!current);
}

#[tauri::command]
async fn zen_summary(state: State<'_, AppState>) -> Result<Vec<String>, AuraError> {
    // This would ideally get the current HTML from the engine
    Ok(vec!["Aura is a sanctuary for focused browsing.".to_string()])
}

#[tauri::command]
async fn silo_status(domain: String, _state: State<'_, AppState>) -> Result<bool, AuraError> {
    Ok(true)
}

pub fn run() {
    let ui = aura_ui::create_ui();
    let hot_swap = Arc::new(HotSwapManager::new());

    let state = AppState {
        hot_swap: hot_swap.clone(),
        ui: ui.clone(),
    };

    // Initialize Silo
    let silo_dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".aura")
        .join("silos");
    let _silo_manager = aura_silo::SiloManager::init(silo_dir).expect("Failed to initialize Silo");

    tauri::Builder::default()
        .manage(state)
        .setup(move |app| {
            // Initial engine load
            let engine_path = if cfg!(windows) {
                "./engines/aura_engine.dll"
            } else if cfg!(target_os = "macos") {
                "./engines/libaura_engine.dylib"
            } else {
                "./engines/libaura_engine.so"
            };

            let hs = hot_swap.clone();
            let path = std::path::PathBuf::from(engine_path);
            tauri::async_runtime::spawn(async move {
                let _ = hs.load_initial_engine(path).await;
            });

            // Set up UI callbacks
            let _ui_handle = ui.as_weak();
            let app_handle = app.handle().clone();
            ui.on_navigate(move |url| {
                let app_handle = app_handle.clone();
                let url = url.to_string();
                tauri::async_runtime::spawn(async move {
                    let state: State<'_, AppState> = app_handle.state();
                    let _ = navigate(url, state).await;
                });
            });

            // Show Slint UI
            ui.show().expect("Failed to show Slint UI");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            navigate,
            toggle_command_bar,
            zen_summary,
            silo_status
        ])
        .run(tauri::generate_context!())
        .expect("Aura failed to launch");
}
