// aura-app/src/lib.rs

pub mod hot_swap;

use aura_ui::{MainUI, TabNode};
use hot_swap::{HotSwapManager, SwapError};
use raw_window_handle::HasWindowHandle;
use serde::Serialize;
use slint::{ComponentHandle, Model, SharedString};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Manager, State};
use thiserror::Error;
use url::Url;

#[derive(Debug, Error, Serialize)]
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
    pub ui: slint::Weak<MainUI>,
    pub ai: Arc<tokio::sync::Mutex<Option<aura_ai::AiEngine>>>,
    pub silo: Arc<aura_silo::SiloManager>,
}

#[tauri::command]
async fn navigate(url: String, state: State<'_, AppState>) -> Result<(), AuraError> {
    let parsed = if url.contains(' ') || (!url.contains('.') && !url.starts_with("localhost")) {
        Url::parse_with_params("https://duckduckgo.com/", &[("q", &url)])
            .map_err(|_| AuraError::InvalidUrl(url.clone()))?
    } else {
        let url_with_scheme = if !url.contains("://") {
            format!("https://{}", url)
        } else {
            url.clone()
        };
        Url::parse(&url_with_scheme).map_err(|_| AuraError::InvalidUrl(url.clone()))?
    };
    let source_url = parsed.clone();
    let filtered = aura_net::intercept(&parsed, &source_url, "main_frame").await;

    match filtered {
        aura_net::InterceptDecision::Allow(target_url)
        | aura_net::InterceptDecision::Redirect(target_url) => {
            state.hot_swap.navigate(target_url.as_str()).await?;

            let ui_weak = state.ui.clone();
            let target_url_str = target_url.to_string();

            let _ = ui_weak.upgrade_in_event_loop({
                let target_url_str = target_url_str.clone();
                move |ui| {
                    ui.set_active_url(target_url_str.into());
                }
            });

            // Try to extract favicon color in background
            let ui_weak_bg = ui_weak.clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(resp) = reqwest::get(format!(
                    "{}/favicon.ico",
                    target_url_str.trim_end_matches('/')
                ))
                .await
                    && let Ok(bytes) = resp.bytes().await
                    && let Some(color) = aura_ui::extract_dominant_color(&bytes)
                {
                    let _ = ui_weak_bg.upgrade_in_event_loop(move |ui| {
                        let tabs = ui.get_tabs();
                        let mut vec: Vec<TabNode> = tabs.iter().collect();
                        for t in &mut vec {
                            if t.active {
                                t.glow_color = color;
                            }
                        }
                        let model = std::rc::Rc::new(slint::VecModel::from(vec));
                        ui.set_tabs(slint::ModelRc::from(model));
                    });
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
async fn toggle_command_bar(state: State<'_, AppState>) -> Result<(), String> {
    state
        .ui
        .upgrade_in_event_loop(|ui| {
            let current = ui.get_command_bar_visible();
            ui.set_command_bar_visible(!current);
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn internal_lotus_clicked(state: &AppState) {
    let ui_weak = state.ui.clone();
    let ai_arc = state.ai.clone();

    let _ = ui_weak.upgrade_in_event_loop(move |ui| {
        let current = ui.get_breathe_visible();
        ui.set_breathe_visible(!current);

        if !current {
            ui.set_status_message("Aura is thinking...".into());
            let ui_weak_inner = ui.as_weak();
            let ai_arc_inner = ai_arc.clone();

            tauri::async_runtime::spawn(async move {
                let mut ai_guard = ai_arc_inner.lock().await;
                if ai_guard.is_none() {
                    *ai_guard = aura_ai::AiEngine::load().await.ok();
                }

                if let Some(ai) = ai_guard.as_mut() {
                    let mock_html = "<html><body><p>Aura is a minimalist browser designed for focus and wellbeing.</p></body></html>";
                    if let Ok(bullets) = ai.summarise(mock_html).await {
                        let _ = ui_weak_inner.upgrade_in_event_loop(move |ui| {
                            let bullets_ss: Vec<SharedString> = bullets.into_iter().map(SharedString::from).collect();
                            let model = std::rc::Rc::new(slint::VecModel::from(bullets_ss));
                            ui.set_breathe_bullets(slint::ModelRc::from(model));
                            ui.set_status_message("Breathe.".into());
                        });
                    }
                } else {
                    let _ = ui_weak_inner.upgrade_in_event_loop(move |ui| {
                        ui.set_status_message("AI offline".into());
                    });
                }
            });
        } else {
            ui.set_status_message("Ready".into());
        }
    });
}

#[tauri::command]
async fn lotus_clicked(state: State<'_, AppState>) -> Result<(), String> {
    internal_lotus_clicked(&state).await;
    Ok(())
}

#[tauri::command]
async fn add_tab(state: State<'_, AppState>, title: String, _url: String) -> Result<(), String> {
    state
        .ui
        .upgrade_in_event_loop(move |ui| {
            let tabs = ui.get_tabs();
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

            let model = std::rc::Rc::new(slint::VecModel::from(vec));
            ui.set_tabs(slint::ModelRc::from(model));
        })
        .map_err(|e| e.to_string())?;
    Ok(())
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
    let ui_weak = ui.as_weak();
    let hot_swap = Arc::new(HotSwapManager::new());
    let ai = Arc::new(tokio::sync::Mutex::new(None));

    let silo_dir = dirs::home_dir()
        .map(|p| p.join(".aura").join("silos"))
        .unwrap_or_else(|| {
            let fallback = std::env::current_dir().unwrap_or_default().join("silos");
            tracing::warn!(
                "Could not find home directory, using fallback: {:?}",
                fallback
            );
            fallback
        });

    if let Err(e) = std::fs::create_dir_all(&silo_dir) {
        tracing::error!("Failed to create silo directory {:?}: {}", silo_dir, e);
    }

    let silo_manager = Arc::new(
        aura_silo::SiloManager::init(silo_dir.clone()).expect("Failed to initialize Silo"),
    );

    let state = AppState {
        hot_swap: hot_swap.clone(),
        ui: ui_weak.clone(),
        ai,
        silo: silo_manager,
    };

    let app = tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            navigate,
            toggle_command_bar,
            zen_summary,
            silo_status,
            lotus_clicked,
            add_tab
        ])
        .setup(|app| {
            // Load engine from resources or exe dir
            let hot_swap = app.state::<AppState>().hot_swap.clone();

            let resource_dir = app.path().resource_dir().unwrap_or_else(|_| PathBuf::new());

            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

            let engine_filename = if cfg!(target_os = "windows") {
                "aura_engine.dll"
            } else if cfg!(target_os = "macos") {
                "libaura_engine.dylib"
            } else {
                "libaura_engine.so"
            };

            let paths_to_check = vec![
                exe_dir.join(engine_filename),
                resource_dir.join("resources").join(engine_filename),
                resource_dir.join(engine_filename),
            ];

            for path in paths_to_check {
                if path.exists() {
                    tracing::info!("Attempting to load engine from {:?}", path);
                    let h = hot_swap.clone();
                    let p = path.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = h.load_initial_engine(p.clone()).await {
                            tracing::error!("Failed to load engine from {:?}: {}", p, e);
                        } else {
                            tracing::info!("Engine loaded successfully from {:?}", p);
                        }
                    });
                    break;
                }
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Aura failed to build");

    let handle = app.handle().clone();

    // Slint callbacks
    let h_nav = handle.clone();
    ui.on_navigate(move |url| {
        let h = h_nav.clone();
        let url = url.to_string();
        tauri::async_runtime::spawn(async move {
            let state: State<'_, AppState> = h.state();
            let _ = navigate(url, state).await;
        });
    });

    let h_lotus = handle.clone();
    ui.on_lotus_clicked(move || {
        let h = h_lotus.clone();
        tauri::async_runtime::spawn(async move {
            let state: State<'_, AppState> = h.state();
            let _ = lotus_clicked(state).await;
        });
    });

    let h_mouse = handle.clone();
    ui.on_mouse_event(move |x, y, event_type| {
        let h = h_mouse.clone();
        tauri::async_runtime::spawn(async move {
            let state: State<'_, AppState> = h.state();
            let _ = state.hot_swap.mouse_event(x, y, event_type).await;
        });
    });

    // Gestural Edge Detection
    let win = app
        .get_webview_window("main")
        .expect("Main window not found in config");
    win.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(_) = event {
            // Example of a valid variant
        }
    });

    ui.set_command_bar_visible(true);
    ui.set_status_bar_visible(true);
    ui.show().expect("Failed to show Slint UI");

    // Start the render loop for Servo
    let h_swap_render = hot_swap.clone();
    let win_render = win.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            let surface = if let Ok(handle) = win_render.window_handle() {
                let raw = handle.as_raw();
                match raw {
                    raw_window_handle::RawWindowHandle::Win32(h) => {
                        h.hwnd.get() as *mut std::ffi::c_void
                    }
                    _ => std::ptr::null_mut(),
                }
            } else {
                std::ptr::null_mut()
            };

            if !surface.is_null() {
                let _ = h_swap_render
                    .paint(hot_swap::SendableSurface(surface))
                    .await;
            }
            tokio::time::sleep(std::time::Duration::from_millis(16)).await; // ~60 FPS
        }
    });

    app.run(|_app_handle, _event| {});
}
