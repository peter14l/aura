// aura-ui/src/lib.rs

slint::include_modules!();

pub use MainUI;

pub fn create_ui() -> MainUI {
    MainUI::new().expect("Failed to initialize MainUI")
}
