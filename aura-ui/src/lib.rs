slint::include_modules!();

pub fn run() {
    let ui = MainUI::new().unwrap();
    
    ui.on_navigate(|url| {
        println!("Navigating to: {}", url);
    });

    ui.run().unwrap();
}
