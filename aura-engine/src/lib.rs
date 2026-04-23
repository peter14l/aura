// aura-engine/src/lib.rs
use std::ffi::{c_char, c_void, CStr};

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
            let url = unsafe { CStr::from_ptr(snapshot.current_url) }
                .to_string_lossy()
                .into_owned();
            self.current_url = url;
        }
        true
    }

    pub fn serialise_state(&self) -> EngineSnapshot {
        // In a real implementation, we'd need to manage the lifecycle of this C string
        let url_ptr = std::ffi::CString::new(self.current_url.clone())
            .unwrap()
            .into_raw();
        EngineSnapshot {
            current_url: url_ptr,
            placeholder: true,
        }
    }

    pub fn navigate(&mut self, url: &str) -> bool {
        self.current_url = url.to_string();
        // Here we would trigger Servo to load the URL
        true
    }

    pub fn release_gpu_surface(&mut self) {}

    pub fn paint_to_surface(&mut self, _surface: *mut c_void) {}
}

#[no_mangle]
pub extern "C" fn aura_engine_version() -> *const c_char {
    c"1.4.2".as_ptr()
}

/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// The caller must ensure that `config` is a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn aura_engine_cold_init(config: *const EngineConfig) -> *mut EngineContext {
    let ctx = Box::new(EngineContext::new_cold(&*config));
    Box::into_raw(ctx)
}

/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `ctx` and `snapshot` are valid pointers.
#[no_mangle]
pub unsafe extern "C" fn aura_engine_warm_init(
    ctx: *mut EngineContext,
    snapshot: *const EngineSnapshot,
) -> bool {
    let ctx = &mut *ctx;
    ctx.restore_from_snapshot(&*snapshot)
}

/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `ctx` and `url` are valid pointers.
#[no_mangle]
pub unsafe extern "C" fn aura_engine_navigate(ctx: *mut EngineContext, url: *const c_char) -> bool {
    if ctx.is_null() || url.is_null() {
        return false;
    }
    let ctx = &mut *ctx;
    let url_str = CStr::from_ptr(url).to_string_lossy();
    ctx.navigate(&url_str)
}

/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `ctx` and `out_snapshot` are valid pointers.
#[no_mangle]
pub unsafe extern "C" fn aura_engine_freeze(
    ctx: *mut EngineContext,
    out_snapshot: *mut EngineSnapshot,
) -> bool {
    let ctx = &mut *ctx;
    let snapshot = ctx.serialise_state();
    *out_snapshot = snapshot;
    ctx.release_gpu_surface();
    true
}

/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `ctx` and `surface` are valid pointers.
#[no_mangle]
pub unsafe extern "C" fn aura_engine_paint(ctx: *mut EngineContext, surface: *mut c_void) {
    let ctx = &mut *ctx;
    ctx.paint_to_surface(surface)
}

/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer and takes ownership of it.
/// The caller must ensure that `ctx` was previously returned by `aura_engine_cold_init`.
#[no_mangle]
pub unsafe extern "C" fn aura_engine_destroy(ctx: *mut EngineContext) {
    if !ctx.is_null() {
        drop(Box::from_raw(ctx))
    }
}
