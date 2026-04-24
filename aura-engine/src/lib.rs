// aura-engine/src/lib.rs
use servo::{Embedder, EmbedderEvent, Servo, ServoOptions};
use std::ffi::{CStr, CString, c_char, c_void};
use url::Url;

/// Opaque handle passed across FFI boundary
pub struct EngineContext {
    current_url: String,
    servo: Servo<AuraEmbedder>,
}

struct AuraEmbedder {
    // 2026 Embedder implementation
}

impl Embedder for AuraEmbedder {
    fn handle_event(&self, event: EmbedderEvent) {
        match event {
            EmbedderEvent::TitleChanged(title) => {
                println!("Aura Engine: Title changed to {}", title);
            }
            EmbedderEvent::UrlChanged(url) => {
                println!("Aura Engine: URL changed to {}", url);
            }
            _ => {}
        }
    }
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
            unsafe {
                CStr::from_ptr(config.user_agent)
                    .to_string_lossy()
                    .into_owned()
            }
        } else {
            "Aura/1.0 (Subtractive Glassmorphism; Rust)".to_string()
        };

        let options = ServoOptions {
            user_agent: ua,
            ..Default::default()
        };

        let servo = Servo::new(AuraEmbedder {}, options);

        Self {
            current_url: String::new(),
            servo,
        }
    }

    pub fn restore_from_snapshot(&mut self, snapshot: &EngineSnapshot) -> bool {
        if !snapshot.current_url.is_null() {
            let url = unsafe { CStr::from_ptr(snapshot.current_url) }
                .to_string_lossy()
                .into_owned();
            self.current_url = url.clone();
            self.servo.load_url(&url);
        }
        true
    }

    pub fn serialise_state(&self) -> EngineSnapshot {
        let url_ptr = CString::new(self.current_url.clone())
            .unwrap_or_else(|_| CString::new("").unwrap())
            .into_raw();
        EngineSnapshot {
            current_url: url_ptr,
            scroll_x: 0.0, // In 2026 we'd query this from Servo
            scroll_y: 0.0,
        }
    }

    pub fn navigate(&mut self, url_str: &str) -> bool {
        if let Ok(_) = Url::parse(url_str) {
            self.current_url = url_str.to_string();
            self.servo.load_url(url_str);
            return true;
        }
        false
    }

    pub fn paint_to_surface(&mut self, surface: *mut c_void) {
        // In 2026, we pass the raw surface handle to Servo's wgpu compositor
        // SAFETY: The shell ensures this surface is valid for the duration of the call
        self.servo.repaint_on_surface(surface);
    }

    pub fn handle_mouse_event(&mut self, x: f32, y: f32, event_type: i32) {
        let event = match event_type {
            0 => servo::MouseEvent::Move,
            1 => servo::MouseEvent::Down(servo::MouseButton::Left),
            2 => servo::MouseEvent::Up(servo::MouseButton::Left),
            _ => return,
        };
        self.servo.handle_mouse_event(x, y, event);
    }
}

#[no_mangle]
pub extern "C" fn aura_engine_version() -> *const c_char {
    c"1.4.2".as_ptr()
}

// ... existing code ...

#[no_mangle]
pub unsafe extern "C" fn aura_engine_mouse_event(
    ctx: *mut EngineContext,
    x: f32,
    y: f32,
    event_type: i32,
) {
    if ctx.is_null() {
        return;
    }
    let ctx = unsafe { &mut *ctx };
    ctx.handle_mouse_event(x, y, event_type);
}

#[no_mangle]
pub unsafe extern "C" fn aura_engine_cold_init(config: *const EngineConfig) -> *mut EngineContext {
    if config.is_null() {
        return std::ptr::null_mut();
    }
    let ctx = Box::new(EngineContext::new_cold(unsafe { &*config }));
    Box::into_raw(ctx)
}

#[no_mangle]
pub unsafe extern "C" fn aura_engine_warm_init(
    ctx: *mut EngineContext,
    snapshot: *const EngineSnapshot,
) -> bool {
    if ctx.is_null() || snapshot.is_null() {
        return false;
    }
    let ctx = unsafe { &mut *ctx };
    ctx.restore_from_snapshot(unsafe { &*snapshot })
}

#[no_mangle]
pub unsafe extern "C" fn aura_engine_navigate(ctx: *mut EngineContext, url: *const c_char) -> bool {
    if ctx.is_null() || url.is_null() {
        return false;
    }
    let ctx = unsafe { &mut *ctx };
    let url_str = unsafe { CStr::from_ptr(url) }.to_string_lossy();
    ctx.navigate(&url_str)
}

#[no_mangle]
pub unsafe extern "C" fn aura_engine_freeze(
    ctx: *mut EngineContext,
    out_snapshot: *mut EngineSnapshot,
) -> bool {
    if ctx.is_null() || out_snapshot.is_null() {
        return false;
    }
    let ctx = unsafe { &mut *ctx };
    let snapshot = ctx.serialise_state();
    unsafe { *out_snapshot = snapshot };
    true
}

#[no_mangle]
pub unsafe extern "C" fn aura_engine_paint(ctx: *mut EngineContext, surface: *mut c_void) {
    if ctx.is_null() {
        return;
    }
    let ctx = unsafe { &mut *ctx };
    ctx.paint_to_surface(surface)
}

#[no_mangle]
pub unsafe extern "C" fn aura_engine_destroy(ctx: *mut EngineContext) {
    if !ctx.is_null() {
        unsafe { drop(Box::from_raw(ctx)) }
    }
}

#[no_mangle]
pub unsafe extern "C" fn aura_engine_free_snapshot(snapshot: *mut EngineSnapshot) {
    if !snapshot.is_null() {
        let s = unsafe { &*snapshot };
        if !s.current_url.is_null() {
            unsafe { drop(CString::from_raw(s.current_url)) };
        }
    }
}
