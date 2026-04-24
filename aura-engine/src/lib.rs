// aura-engine/src/lib.rs
use servo::euclid::Point2D;
use servo::input_events::{
    ElementState, InputEvent, MouseButton, MouseButtonEvent, MouseMoveEvent,
};
use servo::servo_builder::ServoBuilder;
use servo::webview::{WebView, WebViewBuilder};
use std::ffi::{CStr, CString, c_char, c_void};
use std::rc::Rc;
use url::Url;

/// Opaque handle passed across FFI boundary
pub struct EngineContext {
    current_url: String,
    servo: servo::Servo,
    webview: WebView,
}

/// Minimal placeholder for a RenderingContext
struct AuraRenderingContext;
impl servo::rendering_context::RenderingContext for AuraRenderingContext {
    // Implement required methods for painting and buffer management
    // In a real implementation, this would connect to the provided surface
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

        let servo = ServoBuilder::new().user_agent(ua).build();

        // In 2026, WebViewBuilder::new takes the servo instance and a rendering context
        let webview = WebViewBuilder::new(&servo, Rc::new(AuraRenderingContext)).build();

        Self {
            current_url: String::new(),
            servo,
            webview,
        }
    }

    pub fn restore_from_snapshot(&mut self, snapshot: &EngineSnapshot) -> bool {
        if !snapshot.current_url.is_null() {
            let url = unsafe { CStr::from_ptr(snapshot.current_url) }
                .to_string_lossy()
                .into_owned();
            self.current_url = url.clone();
            if let Ok(parsed) = Url::parse(&url) {
                self.webview.load_url(parsed);
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
        if let Ok(url) = Url::parse(url_str) {
            self.current_url = url_str.to_string();
            self.webview.load_url(url);
            return true;
        }
        false
    }

    pub fn paint_to_surface(&mut self, _surface: *mut c_void) {
        // In this architecture, we spin the event loop to trigger repaints
        self.servo.handle_events();
    }

    pub fn handle_mouse_event(&mut self, x: f32, y: f32, event_type: i32) {
        let point = Point2D::new(x, y);
        let event = match event_type {
            0 => InputEvent::MouseMove(MouseMoveEvent {
                point,
                modifiers: Default::default(),
            }),
            1 => InputEvent::MouseButton(MouseButtonEvent {
                button: MouseButton::Left,
                state: ElementState::Pressed,
                point,
                modifiers: Default::default(),
            }),
            2 => InputEvent::MouseButton(MouseButtonEvent {
                button: MouseButton::Left,
                state: ElementState::Released,
                point,
                modifiers: Default::default(),
            }),
            _ => return,
        };
        self.webview.handle_input_event(event);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_engine_version() -> *const c_char {
    c"1.4.2".as_ptr()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aura_engine_cold_init(config: *const EngineConfig) -> *mut EngineContext {
    if config.is_null() {
        return std::ptr::null_mut();
    }
    let ctx = Box::new(EngineContext::new_cold(unsafe { &*config }));
    Box::into_raw(ctx)
}

#[unsafe(no_mangle)]
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aura_engine_navigate(ctx: *mut EngineContext, url: *const c_char) -> bool {
    if ctx.is_null() || url.is_null() {
        return false;
    }
    let ctx = unsafe { &mut *ctx };
    let url_str = unsafe { CStr::from_ptr(url) }.to_string_lossy();
    ctx.navigate(&url_str)
}

#[unsafe(no_mangle)]
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aura_engine_paint(ctx: *mut EngineContext, surface: *mut c_void) {
    if ctx.is_null() {
        return;
    }
    let ctx = unsafe { &mut *ctx };
    ctx.paint_to_surface(surface)
}

#[unsafe(no_mangle)]
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aura_engine_destroy(ctx: *mut EngineContext) {
    if !ctx.is_null() {
        unsafe { drop(Box::from_raw(ctx)) }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aura_engine_free_snapshot(snapshot: *mut EngineSnapshot) {
    if !snapshot.is_null() {
        let s = unsafe { &*snapshot };
        if !s.current_url.is_null() {
            unsafe { drop(CString::from_raw(s.current_url)) };
        }
    }
}
