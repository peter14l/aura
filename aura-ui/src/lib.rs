slint::include_modules!();

pub fn run() {
    let ui = MainUI::new().expect("Failed to initialize MainUI");

    ui.on_navigate(|url| {
        println!("Navigating to: {}", url);
    });

    ui.run().expect("Failed to run Slint event loop");
}
