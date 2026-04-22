// aura-engine/src/lib.rs
use std::ffi::{c_void, c_char, CStr};

/// Opaque handle passed across FFI boundary
pub struct EngineContext {
    current_url: String,
}

#[repr(C)]
pub struct EngineConfig {
    pub placeholder: bool,
}

#[repr(C)]
pub struct EngineSnapshot {
    pub current_url: *mut c_char,
    pub placeholder: bool,
}

impl EngineContext {
    pub fn new_cold(_config: &EngineConfig) -> Self {
        Self {
            current_url: String::new(),
        }
    }

    pub fn restore_from_snapshot(&mut self, snapshot: &EngineSnapshot) -> bool {
        if !snapshot.current_url.is_null() {
            let url = unsafe { CStr::from_ptr(snapshot.current_url) }.to_string_lossy().into_owned();
            self.current_url = url;
        }
        true
    }

    pub fn serialise_state(&self) -> EngineSnapshot {
        // In a real implementation, we'd need to manage the lifecycle of this C string
        let url_ptr = std::ffi::CString::new(self.current_url.clone()).unwrap().into_raw();
        EngineSnapshot { 
            current_url: url_ptr,
            placeholder: true 
        }
    }

    pub fn navigate(&mut self, url: &str) -> bool {
        self.current_url = url.to_string();
        // Here we would trigger Servo to load the URL
        true
    }

    pub fn release_gpu_surface(&mut self) {
    }

    pub fn paint_to_surface(&mut self, _surface: *mut c_void) {
    }
}

#[no_mangle]
pub extern "C" fn aura_engine_version() -> *const u8 {
    b"1.4.2\0".as_ptr()
}

#[no_mangle]
pub extern "C" fn aura_engine_cold_init(config: *const EngineConfig) -> *mut EngineContext {
    let ctx = Box::new(EngineContext::new_cold(unsafe { &*config }));
    Box::into_raw(ctx)
}

#[no_mangle]
pub extern "C" fn aura_engine_warm_init(
    ctx: *mut EngineContext,
    snapshot: *const EngineSnapshot,
) -> bool {
    let ctx = unsafe { &mut *ctx };
    ctx.restore_from_snapshot(unsafe { &*snapshot })
}

#[no_mangle]
pub extern "C" fn aura_engine_navigate(ctx: *mut EngineContext, url: *const c_char) -> bool {
    if ctx.is_null() || url.is_null() { return false; }
    let ctx = unsafe { &mut *ctx };
    let url_str = unsafe { CStr::from_ptr(url) }.to_string_lossy();
    ctx.navigate(&url_str)
}

#[no_mangle]
pub extern "C" fn aura_engine_freeze(
    ctx: *mut EngineContext,
    out_snapshot: *mut EngineSnapshot,
) -> bool {
    let ctx = unsafe { &mut *ctx };
    let snapshot = ctx.serialise_state();
    unsafe { *out_snapshot = snapshot };
    ctx.release_gpu_surface();
    true
}

#[no_mangle]
pub extern "C" fn aura_engine_paint(
    ctx: *mut EngineContext,
    surface: *mut c_void,
) {
    let ctx = unsafe { &mut *ctx };
    ctx.paint_to_surface(surface)
}

#[no_mangle]
pub extern "C" fn aura_engine_destroy(ctx: *mut EngineContext) {
    if !ctx.is_null() {
        unsafe { drop(Box::from_raw(ctx)) }
    }
}
