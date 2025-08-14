use std::thread;
use std::time::Duration;

// This would normally be: use typo_fixer::menu_bar::{setup_menu_bar, get_menu_bar};
// But since we're in the same crate, we'll use the module directly

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Menu Bar Demo");
    println!("Look for the ⌨️ icon in your menu bar!");
    println!("Click on it to see the menu with 'About TypoFixer' and 'Quit TypoFixer' options");
    println!("Use ⌘Q or click 'Quit TypoFixer' to exit");

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Setup menu bar
    typo_fixer::menu_bar::setup_menu_bar()?;

    // Print some info about the demo
    println!("✅ Menu bar icon created successfully!");
    println!("📋 Menu contains:");
    println!("   • About TypoFixer");
    println!("   • ────────────────");
    println!("   • Quit TypoFixer (⌘Q)");

    // Simulate some background work
    thread::spawn(|| loop {
        thread::sleep(Duration::from_secs(30));
        println!("⏰ Background task running... Menu bar is active!");
    });

    // Run the event loop (this blocks until the app terminates)
    let menu_bar = typo_fixer::menu_bar::get_menu_bar();
    let mtm = objc2_foundation::MainThreadMarker::new().expect("must run on main thread");
    menu_bar.lock().unwrap().run_event_loop(mtm);

    Ok(())
}

// Note: This module path needs to be adjusted based on your project structure
mod typo_fixer {
    pub mod menu_bar {
        include!("../src/menu_bar.rs");
    }
}
