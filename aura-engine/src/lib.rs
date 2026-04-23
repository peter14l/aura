// aura-engine/src/lib.rs
use std::ffi::{c_char, c_void, CStr, CString};

/// Opaque handle passed across FFI boundary
pub struct EngineContext {
    current_url: String,
    config: EngineConfigInternal,
}

struct EngineConfigInternal {
    user_agent: String,
    placeholder: bool,
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
        
        Self {
            current_url: String::new(),
            config: EngineConfigInternal {
                user_agent: ua,
                placeholder: config.placeholder,
            },
        }
    }

    pub fn restore_from_snapshot(&mut self, snapshot: &EngineSnapshot) -> bool {
        if !snapshot.current_url.is_null() {
            let url = unsafe { CStr::from_ptr(snapshot.current_url) }
                .to_string_lossy()
                .into_owned();
            self.current_url = url;
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

    pub fn navigate(&mut self, url: &str) -> bool {
        self.current_url = url.to_string();
        // In the future: servo.load_url(url)
        true
    }

    pub fn paint_to_surface(&mut self, _surface: *mut c_void) {
        // Mock paint
    }
}

#[no_mangle]
pub extern "C" fn aura_engine_version() -> *const c_char {
    c"1.4.2".as_ptr()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn aura_engine_cold_init(config: *const EngineConfig) -> *mut EngineContext {
    if config.is_null() { return std::ptr::null_mut(); }
    let ctx = Box::new(EngineContext::new_cold(&*config));
    Box::into_raw(ctx)
}

/// # Safety
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
#[no_mangle]
pub unsafe extern "C" fn aura_engine_navigate(ctx: *mut EngineContext, url: *const c_char) -> bool {
    if ctx.is_null() || url.is_null() { return false; }
    let ctx = &mut *ctx;
    let url_str = CStr::from_ptr(url).to_string_lossy();
    ctx.navigate(&url_str)
}

/// # Safety
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
#[no_mangle]
pub unsafe extern "C" fn aura_engine_paint(ctx: *mut EngineContext, surface: *mut c_void) {
    if ctx.is_null() { return; }
    let ctx = &mut *ctx;
    ctx.paint_to_surface(surface)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn aura_engine_destroy(ctx: *mut EngineContext) {
    if !ctx.is_null() {
        drop(Box::from_raw(ctx))
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn aura_engine_free_snapshot(snapshot: *mut EngineSnapshot) {
    if !snapshot.is_null() {
        let s = &*snapshot;
        if !s.current_url.is_null() {
            drop(CString::from_raw(s.current_url));
        }
    }
}
