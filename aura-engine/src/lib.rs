// aura-engine/src/lib.rs
use std::ffi::{c_char, c_void, CStr, CString};
use std::sync::Arc;
use url::Url;

/// Opaque handle passed across FFI boundary
pub struct EngineContext {
    current_url: String,
    // Servo state
    // Note: Servo initialization is complex and platform-dependent.
    // This implementation sets up the core architecture for embedding Servo.
    servo: Option<Box<ServoInstance>>,
}

struct ServoInstance {
    // In a full implementation, this would hold the Servo instance,
    // the compositor, and the event loop.
    url: String,
}

#[repr(C)]
pub struct EngineConfig {
    pub user_agent: *const c_char,
    pub placeholder: bool,
}

#[repr(C)]
pub struct EngineSnapshot {
    pub current_url: *mut c_char,
    pub scroll_x: f32,
    pub scroll_y: f32,
}

impl EngineContext {
    pub fn new_cold(config: &EngineConfig) -> Self {
        let ua = if !config.user_agent.is_null() {
            unsafe { CStr::from_ptr(config.user_agent).to_string_lossy().into_owned() }
        } else {
            "Aura/1.0".to_string()
        };

        // Initialize Servo components here
        // For the cdylib, we establish the bridge to the servo crate
        
        Self {
            current_url: String::new(),
            servo: Some(Box::new(ServoInstance {
                url: String::new(),
            })),
        }
    }

    pub fn restore_from_snapshot(&mut self, snapshot: &EngineSnapshot) -> bool {
        if !snapshot.current_url.is_null() {
            let url = unsafe { CStr::from_ptr(snapshot.current_url) }
                .to_string_lossy()
                .into_owned();
            self.current_url = url.clone();
            if let Some(ref mut instance) = self.servo {
                instance.url = url;
            }
        }
        true
    }

    pub fn serialise_state(&self) -> EngineSnapshot {
        let url_ptr = CString::new(self.current_url.clone())
            .unwrap_or_else(|_| CString::new("").unwrap())
            .into_raw();
        EngineSnapshot {
            current_url: url_ptr,
            scroll_x: 0.0,
            scroll_y: 0.0,
        }
    }

    pub fn navigate(&mut self, url_str: &str) -> bool {
        self.current_url = url_str.to_string();
        if let Ok(url) = Url::parse(url_str) {
            if let Some(ref mut instance) = self.servo {
                instance.url = url.to_string();
                // Real implementation: servo.load_url(url)
                return true;
            }
        }
        false
    }

    pub fn paint_to_surface(&mut self, _surface: *mut c_void) {
        // Here we would hook into Servo's compositor to render
        // to the platform-specific surface (HWND, NSView, etc.)
    }
}

#[no_mangle]
pub extern "C" fn aura_engine_version() -> *const c_char {
    c"1.4.2".as_ptr()
}

/// # Safety
/// Caller must ensure config is a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn aura_engine_cold_init(config: *const EngineConfig) -> *mut EngineContext {
    if config.is_null() { return std::ptr::null_mut(); }
    let ctx = Box::new(EngineContext::new_cold(&*config));
    Box::into_raw(ctx)
}

/// # Safety
/// Caller must ensure ctx and snapshot are valid pointers.
#[no_mangle]
pub unsafe extern "C" fn aura_engine_warm_init(
    ctx: *mut EngineContext,
    snapshot: *const EngineSnapshot,
) -> bool {
    if ctx.is_null() || snapshot.is_null() { return false; }
    let ctx = &mut *ctx;
    ctx.restore_from_snapshot(&*snapshot)
}

/// # Safety
/// Caller must ensure ctx and url are valid pointers.
#[no_mangle]
pub unsafe extern "C" fn aura_engine_navigate(ctx: *mut EngineContext, url: *const c_char) -> bool {
    if ctx.is_null() || url.is_null() { return false; }
    let ctx = &mut *ctx;
    let url_str = CStr::from_ptr(url).to_string_lossy();
    ctx.navigate(&url_str)
}

/// # Safety
/// Caller must ensure ctx and out_snapshot are valid pointers.
#[no_mangle]
pub unsafe extern "C" fn aura_engine_freeze(
    ctx: *mut EngineContext,
    out_snapshot: *mut EngineSnapshot,
) -> bool {
    if ctx.is_null() || out_snapshot.is_null() { return false; }
    let ctx = &mut *ctx;
    let snapshot = ctx.serialise_state();
    *out_snapshot = snapshot;
    true
}

/// # Safety
/// Caller must ensure ctx and surface are valid pointers.
#[no_mangle]
pub unsafe extern "C" fn aura_engine_paint(ctx: *mut EngineContext, surface: *mut c_void) {
    if ctx.is_null() { return; }
    let ctx = &mut *ctx;
    ctx.paint_to_surface(surface)
}

/// # Safety
/// Caller must take ownership of the returned pointer.
#[no_mangle]
pub unsafe extern "C" fn aura_engine_destroy(ctx: *mut EngineContext) {
    if !ctx.is_null() {
        drop(Box::from_raw(ctx))
    }
}

/// # Safety
/// Caller must ensure snapshot is a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn aura_engine_free_snapshot(snapshot: *mut EngineSnapshot) {
    if !snapshot.is_null() {
        let s = &*snapshot;
        if !s.current_url.is_null() {
            drop(CString::from_raw(s.current_url));
        }
    }
}
