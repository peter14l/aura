// aura-app/src/hot_swap.rs

use libloading::{Library, Symbol};
use std::ffi::{CString, c_char, c_void};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Error)]
pub enum SwapError {
    #[error("Init failed")]
    InitFailed,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Library error: {0}")]
    Library(#[from] libloading::Error),
    #[error("Network error: {0}")]
    Net(#[from] reqwest::Error),
}

// ABI types mirrored from aura-engine
#[repr(C)]
pub struct EngineContext;

#[repr(C)]
pub struct EngineConfig {
    pub user_agent: *const c_char,
    pub placeholder: bool,
    pub window_handle: *mut c_void,
    pub display_handle: *mut c_void,
    pub platform: u32,
}

#[repr(C)]
pub struct EngineSnapshot {
    pub current_url: *mut c_char,
    pub placeholder: bool,
}

/// Safety wrapper for passing surface pointers across threads
pub struct SendableSurface(pub *mut c_void);
unsafe impl Send for SendableSurface {}

pub struct LoadedEngine {
    lib: Library,
    ctx: *mut EngineContext,
}

// We need to implement Send and Sync for LoadedEngine because it's wrapped in a Mutex/Arc
// but libloading::Library and raw pointers aren't Send/Sync by default.
// SAFETY: The engine context is managed by our shell and we ensure serialized access via Mutex.
unsafe impl Send for LoadedEngine {}
unsafe impl Sync for LoadedEngine {}

impl LoadedEngine {
    pub fn navigate(&self, url: &str) -> Result<(), SwapError> {
        unsafe {
            let navigate_fn: Symbol<
                unsafe extern "C" fn(*mut EngineContext, *const c_char) -> bool,
            > = self.lib.get(b"aura_engine_navigate")?;
            let c_url = CString::new(url).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid URL string")
            })?;
            if !navigate_fn(self.ctx, c_url.as_ptr()) {
                return Err(SwapError::InitFailed);
            }
        }
        Ok(())
    }

    pub fn paint(&self, surface: *mut c_void) -> Result<(), SwapError> {
        unsafe {
            let paint_fn: Symbol<unsafe extern "C" fn(*mut EngineContext, *mut c_void)> =
                self.lib.get(b"aura_engine_paint")?;
            paint_fn(self.ctx, surface);
        }
        Ok(())
    }

    pub fn mouse_event(&self, x: f32, y: f32, event_type: i32) -> Result<(), SwapError> {
        unsafe {
            let mouse_fn: Symbol<unsafe extern "C" fn(*mut EngineContext, f32, f32, i32)> =
                self.lib.get(b"aura_engine_mouse_event")?;
            mouse_fn(self.ctx, x, y, event_type);
        }
        Ok(())
    }

    pub fn resize(&self, width: u32, height: u32) -> Result<(), SwapError> {
        unsafe {
            if let Ok(resize_fn) = self
                .lib
                .get::<unsafe extern "C" fn(*mut EngineContext, u32, u32)>(b"aura_engine_resize")
            {
                resize_fn(self.ctx, width, height);
            }
        }
        Ok(())
    }
}

impl Drop for LoadedEngine {
    fn drop(&mut self) {
        unsafe {
            if let Ok(destroy_fn) = self
                .lib
                .get::<unsafe extern "C" fn(*mut EngineContext)>(b"aura_engine_destroy")
            {
                destroy_fn(self.ctx);
            }
        }
    }
}

pub struct HotSwapManager {
    current: Arc<Mutex<Option<LoadedEngine>>>,
    // Store handles for shadow instance initialization
    handles: Mutex<Option<(SendableSurface, SendableSurface, u32)>>,
}

impl Default for HotSwapManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HotSwapManager {
    pub fn new() -> Self {
        Self {
            current: Arc::new(Mutex::new(None)),
            handles: Mutex::new(None),
        }
    }

    pub async fn load_initial_engine(
        &self,
        path: PathBuf,
        w_ptr: SendableSurface,
        d_ptr: SendableSurface,
        platform: u32,
    ) -> Result<(), SwapError> {
        {
            let mut h_guard = self.handles.lock().await;
            *h_guard = Some((SendableSurface(w_ptr.0), SendableSurface(d_ptr.0), platform));
        }
        let engine = self.load_engine(path, w_ptr, d_ptr, platform).await?;
        let mut guard = self.current.lock().await;
        *guard = Some(engine);
        Ok(())
    }

    async fn load_engine(
        &self,
        path: PathBuf,
        w_ptr: SendableSurface,
        d_ptr: SendableSurface,
        platform: u32,
    ) -> Result<LoadedEngine, SwapError> {
        unsafe {
            let lib = Library::new(&path)?;
            let cold_init: Symbol<unsafe extern "C" fn(*const EngineConfig) -> *mut EngineContext> =
                lib.get(b"aura_engine_cold_init")?;

            let config = EngineConfig {
                user_agent: std::ptr::null(),
                placeholder: true,
                window_handle: w_ptr.0,
                display_handle: d_ptr.0,
                platform,
            };
            let ctx = cold_init(&config);

            if ctx.is_null() {
                return Err(SwapError::InitFailed);
            }

            Ok(LoadedEngine { lib, ctx })
        }
    }

    pub async fn navigate(&self, url: &str) -> Result<(), SwapError> {
        let guard = self.current.lock().await;
        if let Some(engine) = guard.as_ref() {
            engine.navigate(url)
        } else {
            Err(SwapError::InitFailed)
        }
    }

    pub async fn paint(&self, surface: SendableSurface) -> Result<(), SwapError> {
        let guard = self.current.lock().await;
        if let Some(engine) = guard.as_ref() {
            engine.paint(surface.0)
        } else {
            Err(SwapError::InitFailed)
        }
    }

    pub async fn mouse_event(&self, x: f32, y: f32, event_type: i32) -> Result<(), SwapError> {
        let guard = self.current.lock().await;
        if let Some(engine) = guard.as_ref() {
            engine.mouse_event(x, y, event_type)
        } else {
            Err(SwapError::InitFailed)
        }
    }

    pub async fn resize(&self, width: u32, height: u32) -> Result<(), SwapError> {
        let guard = self.current.lock().await;
        if let Some(engine) = guard.as_ref() {
            engine.resize(width, height)
        } else {
            Err(SwapError::InitFailed)
        }
    }

    pub async fn perform_handoff(&self, new_dylib: PathBuf) -> Result<(), SwapError> {
        // Phase A: Load shadow instance
        let (w, d, p) = {
            let h_guard = self.handles.lock().await;
            let (w, d, p) = h_guard.as_ref().ok_or(SwapError::InitFailed)?;
            (w.0, d.0, *p)
        };
        let new_engine = self.load_engine(new_dylib, SendableSurface(w), SendableSurface(d), p).await?;

        // Phase B: Serialise state
        let mut guard = self.current.lock().await;
        if let Some(old_engine) = guard.as_ref() {
            unsafe {
                let mut snapshot = EngineSnapshot {
                    current_url: std::ptr::null_mut(),
                    placeholder: true,
                };
                let freeze_fn: Symbol<
                    unsafe extern "C" fn(*mut EngineContext, *mut EngineSnapshot) -> bool,
                > = old_engine.lib.get(b"aura_engine_freeze")?;
                freeze_fn(old_engine.ctx, &mut snapshot);

                // Phase C: Swap
                // (In a real implementation, we'd signal the compositor here)

                let warm_init: Symbol<
                    unsafe extern "C" fn(*mut EngineContext, *const EngineSnapshot) -> bool,
                > = new_engine.lib.get(b"aura_engine_warm_init")?;
                warm_init(new_engine.ctx, &snapshot);

                // Clean up snapshot
                if let Ok(free_fn) = old_engine
                    .lib
                    .get::<unsafe extern "C" fn(*mut EngineSnapshot)>(b"aura_engine_free_snapshot")
                {
                    free_fn(&mut snapshot);
                }
            }
        }

        // Finalize swap
        *guard = Some(new_engine);

        Ok(())
    }
}
