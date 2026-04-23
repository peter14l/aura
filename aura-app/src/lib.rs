// aura-app/src/lib.rs

pub mod hot_swap;

use aura_ui::{MainUI, TabNode};
use hot_swap::{HotSwapManager, SwapError};
use slint::{ComponentHandle, Model};
use std::sync::Arc;
use tauri::{Manager, State};
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
    pub ai: Arc<tokio::sync::Mutex<Option<aura_ai::AiEngine>>>,
    pub silo: Arc<aura_silo::SiloManager>,
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
            state.ui.set_active_url(target_url.as_str().into());

            // Try to extract favicon color in background
            let ui_handle = state.ui.clone();
            let target_url_str = target_url.to_string();
            tauri::async_runtime::spawn(async move {
                if let Ok(resp) = reqwest::get(format!(
                    "{}/favicon.ico",
                    target_url_str.trim_end_matches('/')
                ))
                .await
                {
                    if let Ok(bytes) = resp.bytes().await {
                        if let Some(color) = aura_ui::extract_dominant_color(&bytes) {
                            let tabs = ui_handle.get_tabs();
                            let mut vec: Vec<TabNode> = tabs.iter().collect();
                            for t in &mut vec {
                                if t.active {
                                    t.glow_color = color;
                                }
                            }
                            let model = std::sync::Arc::new(slint::VecModel::from(vec));
                            ui_handle.set_tabs(model.into());
                        }
                    }
                }
            });

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

async fn internal_lotus_clicked(state: &AppState) {
    let current = state.ui.get_breathe_visible();
    state.ui.set_breathe_visible(!current);

    if !current {
        state.ui.set_status_message("Aura is thinking...".into());
        let ai_arc = state.ai.clone();
        let ui_handle = state.ui.clone();

        tauri::async_runtime::spawn(async move {
            let mut ai_guard = ai_arc.lock().await;
            if ai_guard.is_none() {
                *ai_guard = aura_ai::AiEngine::load().await.ok();
            }

            if let Some(ai) = ai_guard.as_mut() {
                let mock_html = "<html><body><p>Aura is a minimalist browser designed for focus and wellbeing.</p></body></html>";
                if let Ok(bullets) = ai.summarise(mock_html).await {
                    let model = std::sync::Arc::new(slint::VecModel::from(bullets));
                    ui_handle.set_breathe_bullets(model.into());
                    ui_handle.set_status_message("Breathe.".into());
                }
            } else {
                ui_handle.set_status_message("AI offline".into());
            }
        });
    } else {
        state.ui.set_status_message("Ready".into());
    }
}

#[tauri::command]
async fn lotus_clicked(state: State<'_, AppState>) {
    internal_lotus_clicked(&state).await;
}

#[tauri::command]
fn add_tab(state: State<'_, AppState>, title: String, _url: String) {
    let tabs = state.ui.get_tabs();
    let new_id = tabs.iter().map(|t| t.id).max().unwrap_or(0) + 1;

    let glow_color = slint::Color::from_rgb_u8(212, 225, 209);

    let node = TabNode {
        id: new_id,
        title: title.into(),
        favicon: slint::Image::default(),
        glow_color,
        active: true,
        pinned: false,
    };

    let mut vec: Vec<TabNode> = tabs.iter().collect();
    for t in &mut vec {
        t.active = false;
    }
    vec.push(node);

    let model = std::sync::Arc::new(slint::VecModel::from(vec));
    state.ui.set_tabs(model.into());
}

#[tauri::command]
async fn zen_summary(state: State<'_, AppState>) -> Result<Vec<String>, AuraError> {
    let ai_arc = state.ai.clone();
    let mut ai_guard = ai_arc.lock().await;

    if ai_guard.is_none() {
        *ai_guard = aura_ai::AiEngine::load().await.ok();
    }

    if let Some(ai) = ai_guard.as_mut() {
        // In a real app, get HTML from engine
        let mock_html = "<html><body><p>Aura is a minimalist browser designed for focus and wellbeing.</p></body></html>";
        ai.summarise(mock_html)
            .await
            .map_err(|e| AuraError::EngineError(e.to_string()))
    } else {
        Err(AuraError::EngineError("AI Engine failed to load".into()))
    }
}

#[tauri::command]
async fn silo_status(domain: String, state: State<'_, AppState>) -> Result<bool, AuraError> {
    // A domain is "secured" if we can open its silo
    match state.silo.open_silo(&domain) {
        Ok(_) => Ok(true),
        Err(e) => Err(AuraError::EngineError(format!("Silo error: {}", e))),
    }
}

pub fn run() {
    let ui = aura_ui::create_ui();
    let hot_swap = Arc::new(HotSwapManager::new());
    let ai = Arc::new(tokio::sync::Mutex::new(None));

    let silo_dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".aura")
        .join("silos");
    std::fs::create_dir_all(&silo_dir).expect("Failed to create silo directory");
    let silo_manager =
        Arc::new(aura_silo::SiloManager::init(silo_dir).expect("Failed to initialize Silo"));

    let state = AppState {
        hot_swap: hot_swap.clone(),
        ui: ui.clone(),
        ai,
        silo: silo_manager,
    };

    tauri::Builder::default()
        .manage(state)
        .setup(move |app| {
            tauri::async_runtime::spawn(async move {
                aura_net::init_adblock(&[
                    "https://easylist.to/easylist/easylist.txt",
                    "https://easylist.to/easylist/easyprivacy.txt",
                ])
                .await;
            });

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

            // Gestural Edge Detection
            let win = app
                .get_webview_window("main")
                .expect("Main window not found");
            let ui_gestures = ui.as_weak();
            win.on_window_event(move |event| {
                if let tauri::WindowEvent::CursorMoved { position, .. } = event {
                    if let Some(ui) = ui_gestures.upgrade() {
                        let x = position.x;
                        let y = position.y;

                        ui.set_constellation_visible(x < 60.0);
                        ui.set_address_ghost_visible(y < 60.0);
                        ui.set_status_bar_visible(y > 740.0);
                    }
                }
            });

            let app_handle = app.handle().clone();
            ui.on_navigate(move |url| {
                let app_handle = app_handle.clone();
                let url = url.to_string();
                tauri::async_runtime::spawn(async move {
                    let state: State<'_, AppState> = app_handle.state();
                    let _ = navigate(url, state).await;
                });
            });

            let app_handle_lotus = app.handle().clone();
            ui.on_lotus_clicked(move || {
                let app_handle = app_handle_lotus.clone();
                tauri::async_runtime::spawn(async move {
                    let state: State<'_, AppState> = app_handle.state();
                    internal_lotus_clicked(&state).await;
                });
            });

            ui.show().expect("Failed to show Slint UI");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            navigate,
            toggle_command_bar,
            zen_summary,
            silo_status,
            lotus_clicked,
            add_tab
        ])
        .run(tauri::generate_context!())
        .expect("Aura failed to launch");
}
