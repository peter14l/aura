// aura-engine/src/lib.rs
use euclid::Box2D;
use servo::input_events::{
    InputEvent, MouseButton, MouseButtonAction, MouseButtonEvent, MouseMoveEvent,
};
use servo::{RenderingContext, ServoBuilder, WebView, WebViewBuilder};
use std::ffi::{CStr, CString, c_char, c_void};
use std::rc::Rc;
use url::Url;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{
    ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext,
    PossiblyCurrentGlContext, Version,
};
use glutin::display::{Display, DisplayApiPreference, GlDisplay};
use glutin::surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface};
use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, RawDisplayHandle, RawWindowHandle,
    WaylandDisplayHandle, WaylandWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
    XlibDisplayHandle, XlibWindowHandle,
};
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

/// Opaque handle passed across FFI boundary
pub struct EngineContext {
    current_url: String,
    #[allow(dead_code)]
    servo: servo::Servo,
    webview: WebView,
}

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
    pub scroll_x: f32,
    pub scroll_y: f32,
}

struct GlContext {
    display: Display,
    context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
    glow: std::sync::Arc<glow::Context>,
    gleam: std::rc::Rc<dyn gleam::gl::Gl>,
}

impl GlContext {
    pub fn new(window_handle: RawWindowHandle, display_handle: RawDisplayHandle) -> Self {
        #[cfg(target_os = "windows")]
        let pref = DisplayApiPreference::Wgl(Some(window_handle));
        #[cfg(target_os = "macos")]
        let pref = DisplayApiPreference::Cgl;
        #[cfg(all(unix, not(target_os = "macos")))]
        let pref = DisplayApiPreference::Egl;

        let display = unsafe { Display::new(display_handle, pref) }.unwrap_or_else(|_| {
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                unsafe {
                    Display::new(
                        display_handle,
                        DisplayApiPreference::Glx(Box::new(|_| {})),
                    )
                }
                .expect("Failed to create Glutin Display")
            }
            #[cfg(not(all(unix, not(target_os = "macos"))))]
            {
                panic!("Failed to create Glutin Display")
            }
        });

        let template = ConfigTemplateBuilder::new().build();
        let config = unsafe { display.find_configs(template) }
            .unwrap()
            .next()
            .unwrap();

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .build(Some(window_handle));

        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(Some(Version::new(2, 0))))
            .build(Some(window_handle));

        let not_current_context = unsafe {
            display
                .create_context(&config, &context_attributes)
                .unwrap_or_else(|_| {
                    display
                        .create_context(&config, &fallback_context_attributes)
                        .unwrap()
                })
        };

        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            window_handle,
            NonZeroU32::new(1024).unwrap(),
            NonZeroU32::new(768).unwrap(),
        );

        let gl_surface = unsafe { display.create_window_surface(&config, &attrs).unwrap() };

        let gl_context = not_current_context.make_current(&gl_surface).unwrap();

        let glow_context = unsafe {
            glow::Context::from_loader_function(|s| {
                let s = std::ffi::CString::new(s).unwrap();
                display.get_proc_address(s.as_c_str())
            })
        };

        let gleam_context = unsafe {
            gleam::gl::GlFns::load_with(|s| {
                let s = std::ffi::CString::new(s).unwrap();
                display.get_proc_address(s.as_c_str())
            })
        };

        Self {
            display,
            context: gl_context,
            surface: gl_surface,
            glow: std::sync::Arc::new(glow_context),
            gleam: gleam_context,
        }
    }

    pub fn make_current(&self) {
        self.context.make_current(&self.surface).unwrap();
    }

    pub fn present(&self) {
        self.surface.swap_buffers(&self.context).unwrap();
    }

    pub fn resize(&mut self, width: std::num::NonZeroU32, height: std::num::NonZeroU32) {
        self.surface.resize(&self.context, width, height);
    }
}

/// Real implementation for a RenderingContext
struct AuraRenderingContext {
    gl_context: Arc<Mutex<Option<GlContext>>>,
    size: Arc<Mutex<dpi::PhysicalSize<u32>>>,
}

impl RenderingContext for AuraRenderingContext {
    fn read_to_image(
        &self,
        _: Box2D<i32, servo::DevicePixel>,
    ) -> Option<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>> {
        None
    }
    fn size(&self) -> dpi::PhysicalSize<u32> {
        *self.size.lock().unwrap()
    }
    fn resize(&self, new_size: dpi::PhysicalSize<u32>) {
        *self.size.lock().unwrap() = new_size;
        let mut guard = self.gl_context.lock().unwrap();
        if let Some(ctx) = guard.as_mut() {
            if let (Some(w), Some(h)) = (
                std::num::NonZeroU32::new(new_size.width),
                std::num::NonZeroU32::new(new_size.height),
            ) {
                ctx.resize(w, h);
            }
        }
    }
    fn present(&self) {
        let guard = self.gl_context.lock().unwrap();
        if let Some(ctx) = guard.as_ref() {
            ctx.present();
        }
    }
    fn make_current(&self) -> Result<(), surfman::Error> {
        let guard = self.gl_context.lock().unwrap();
        if let Some(ctx) = guard.as_ref() {
            ctx.make_current();
        }
        Ok(())
    }
    fn gleam_gl_api(&self) -> Rc<dyn gleam::gl::Gl> {
        let guard = self.gl_context.lock().unwrap();
        guard.as_ref().unwrap().gleam.clone()
    }
    fn glow_gl_api(&self) -> std::sync::Arc<glow::Context> {
        let guard = self.gl_context.lock().unwrap();
        guard.as_ref().unwrap().glow.clone()
    }
}

fn reconstruct_handles(config: &EngineConfig) -> (RawWindowHandle, RawDisplayHandle) {
    match config.platform {
        0 => {
            // Windows
            let w = Win32WindowHandle::new(
                std::num::NonZeroIsize::new(config.window_handle as isize).expect("Invalid HWND"),
            );
            (
                RawWindowHandle::Win32(w),
                RawDisplayHandle::Windows(WindowsDisplayHandle::new()),
            )
        }
        1 => {
            // macOS
            let w =
                AppKitWindowHandle::new(std::ptr::NonNull::new(config.window_handle).unwrap());
            (
                RawWindowHandle::AppKit(w),
                RawDisplayHandle::AppKit(AppKitDisplayHandle::new()),
            )
        }
        2 => {
            // X11
            let w = XlibWindowHandle::new(config.window_handle as u32 as u64);
            let d = XlibDisplayHandle::new(std::ptr::NonNull::new(config.display_handle), 0);
            (RawWindowHandle::Xlib(w), RawDisplayHandle::Xlib(d))
        }
        3 => {
            // Wayland
            let w =
                WaylandWindowHandle::new(std::ptr::NonNull::new(config.window_handle).unwrap());
            let d =
                WaylandDisplayHandle::new(std::ptr::NonNull::new(config.display_handle).unwrap());
            (RawWindowHandle::Wayland(w), RawDisplayHandle::Wayland(d))
        }
        _ => panic!("Unsupported platform"),
    }
}

impl EngineContext {
    pub fn new_cold(config: &EngineConfig) -> Self {
        let _ua = if !config.user_agent.is_null() {
            unsafe {
                CStr::from_ptr(config.user_agent)
                    .to_string_lossy()
                    .into_owned()
            }
        } else {
            "Aura/1.0 (Subtractive Glassmorphism; Rust)".to_string()
        };

        let servo = ServoBuilder::default().build();

        let gl_context = if config.window_handle.is_null() {
            None
        } else {
            let (wh, dh) = reconstruct_handles(config);
            Some(GlContext::new(wh, dh))
        };

        let rendering_context = AuraRenderingContext {
            gl_context: Arc::new(Mutex::new(gl_context)),
            size: Arc::new(Mutex::new(dpi::PhysicalSize::new(1024, 768))),
        };

        let webview = WebViewBuilder::new(&servo, Rc::new(rendering_context)).build();

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
                self.webview.load(parsed);
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
            self.webview.load(url);
            return true;
        }
        false
    }

    pub fn paint_to_surface(&mut self, _surface: *mut c_void) {
        // Trigger paint
        self.webview.paint();
    }

    pub fn handle_mouse_event(&mut self, x: f32, y: f32, event_type: i32) {
        let point = euclid::Point2D::new(x, y);
        let webview_point = servo::WebViewPoint::Device(point);

        let event = match event_type {
            0 => InputEvent::MouseMove(MouseMoveEvent {
                point: webview_point,
                is_compatibility_event_for_touch: false,
            }),
            1 => InputEvent::MouseButton(MouseButtonEvent {
                button: MouseButton::Left,
                action: MouseButtonAction::Down,
                point: webview_point,
            }),
            2 => InputEvent::MouseButton(MouseButtonEvent {
                button: MouseButton::Left,
                action: MouseButtonAction::Up,
                point: webview_point,
            }),
            _ => return,
        };
        self.webview.notify_input_event(event);
    }
}

/// Get the version of the engine.
#[unsafe(no_mangle)]
pub extern "C" fn aura_engine_version() -> *const c_char {
    c"1.4.2".as_ptr()
}

/// Initialize the engine from a configuration.
///
/// # Safety
/// The `config` pointer must be a valid, non-null pointer to an `EngineConfig` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aura_engine_cold_init(config: *const EngineConfig) -> *mut EngineContext {
    if config.is_null() {
        return std::ptr::null_mut();
    }
    let ctx = Box::new(EngineContext::new_cold(unsafe { &*config }));
    Box::into_raw(ctx)
}

/// Restore the engine from a snapshot.
///
/// # Safety
/// The `ctx` pointer must be a valid, non-null pointer to an `EngineContext` struct.
/// The `snapshot` pointer must be a valid, non-null pointer to an `EngineSnapshot` struct.
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

/// Navigate to a URL.
///
/// # Safety
/// The `ctx` pointer must be a valid, non-null pointer to an `EngineContext` struct.
/// The `url` pointer must be a valid, non-null C-style string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aura_engine_navigate(ctx: *mut EngineContext, url: *const c_char) -> bool {
    if ctx.is_null() || url.is_null() {
        return false;
    }
    let ctx = unsafe { &mut *ctx };
    let url_str = unsafe { CStr::from_ptr(url) }.to_string_lossy();
    ctx.navigate(&url_str)
}

/// Freeze the current state of the engine into a snapshot.
///
/// # Safety
/// The `ctx` pointer must be a valid, non-null pointer to an `EngineContext` struct.
/// The `out_snapshot` pointer must be a valid, non-null pointer to an `EngineSnapshot` struct.
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

/// Paint the current state of the engine to a surface.
///
/// # Safety
/// The `ctx` pointer must be a valid, non-null pointer to an `EngineContext` struct.
/// The `surface` pointer must be a valid pointer to a rendering surface compatible with the engine.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aura_engine_paint(ctx: *mut EngineContext, surface: *mut c_void) {
    if ctx.is_null() {
        return;
    }
    let ctx = unsafe { &mut *ctx };
    ctx.paint_to_surface(surface)
}

/// Send a mouse event to the engine.
///
/// # Safety
/// The `ctx` pointer must be a valid, non-null pointer to an `EngineContext` struct.
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

/// Send a resize event to the engine.
///
/// # Safety
/// The `ctx` pointer must be a valid, non-null pointer to an `EngineContext` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aura_engine_resize(ctx: *mut EngineContext, width: u32, height: u32) {
    if ctx.is_null() {
        return;
    }
    let ctx = unsafe { &mut *ctx };
    let size = dpi::PhysicalSize::new(width, height);
    ctx.webview.resize(size);
}

/// Destroy the engine context and free its memory.
///
/// # Safety
/// The `ctx` pointer must be a valid pointer to an `EngineContext` struct previously returned by `aura_engine_cold_init`.
/// After calling this, the pointer is no longer valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aura_engine_destroy(ctx: *mut EngineContext) {
    if !ctx.is_null() {
        unsafe { drop(Box::from_raw(ctx)) }
    }
}

/// Free a snapshot previously returned by `aura_engine_freeze`.
///
/// # Safety
/// The `snapshot` pointer must be a valid pointer to an `EngineSnapshot` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aura_engine_free_snapshot(snapshot: *mut EngineSnapshot) {
    if !snapshot.is_null() {
        let s = unsafe { &*snapshot };
        if !s.current_url.is_null() {
            unsafe { drop(CString::from_raw(s.current_url)) };
        }
    }
}
