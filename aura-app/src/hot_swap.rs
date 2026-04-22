// aura-app/src/hot_swap.rs

use libloading::{Library, Symbol};
use std::ffi::{c_char, c_void, CString};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, thiserror::Error)]
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
    pub placeholder: bool,
}

#[repr(C)]
pub struct EngineSnapshot {
    pub current_url: *mut c_char,
    pub placeholder: bool,
}

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
}

impl HotSwapManager {
    pub fn new() -> Self {
        Self {
            current: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn load_initial_engine(&self, path: PathBuf) -> Result<(), SwapError> {
        let engine = self.load_engine(path).await?;
        let mut guard = self.current.lock().await;
        *guard = Some(engine);
        Ok(())
    }

    async fn load_engine(&self, path: PathBuf) -> Result<LoadedEngine, SwapError> {
        unsafe {
            let lib = Library::new(&path)?;
            let cold_init: Symbol<unsafe extern "C" fn(*const EngineConfig) -> *mut EngineContext> =
                lib.get(b"aura_engine_cold_init")?;

            let config = EngineConfig { placeholder: true };
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
}
