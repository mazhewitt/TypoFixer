use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{define_class, msg_send, sel, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, 
    NSMenu, NSMenuItem, NSStatusBar, NSVariableStatusItemLength,
    NSImage, NSAlert, NSCellImagePosition,
};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSString,
    NSAutoreleasePool, ns_string,
};
use tracing::info;

// Instance variables for our custom AppDelegate class
#[derive(Debug, Default)]
struct AppDelegateIvars {}

// Define the AppDelegate class that handles the menu bar
define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = AppDelegateIvars]
    struct AppDelegate;

    unsafe impl NSObjectProtocol for AppDelegate {}

    unsafe impl NSApplicationDelegate for AppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self, _notification: &NSNotification) {
            let mtm = unsafe { MainThreadMarker::new_unchecked() };
            self.setup_status_bar(mtm);
        }

        #[unsafe(method(showAbout:))]
        fn show_about(&self, _sender: *const NSObject) {
            unsafe { show_about_dialog(); }
        }

        #[unsafe(method(quitApp:))]
        fn quit_app(&self, _sender: *const NSObject) {
            let app = NSApplication::sharedApplication(unsafe { MainThreadMarker::new_unchecked() });
            unsafe { app.terminate(None); }
        }
    }
);

impl AppDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(AppDelegateIvars::default());
        unsafe { msg_send![super(this), init] }
    }

    fn setup_status_bar(&self, mtm: MainThreadMarker) {
        info!("Setting up status bar...");
        
        let status_bar = unsafe { NSStatusBar::systemStatusBar() };
        info!("Got system status bar");
        
        let status_item = unsafe { 
            status_bar.statusItemWithLength(NSVariableStatusItemLength)
        };
        info!("Created status item");

        // Create an SF Symbol image for the status bar icon
        let image = unsafe {
            NSImage::imageWithSystemSymbolName_accessibilityDescription(
                &NSString::from_str("keyboard.fill"),
                None,
            )
        };
        
        if let Some(image) = image {
            unsafe { image.setTemplate(true); }
            info!("Created template image");
            
            let button = unsafe { status_item.button(mtm) };
            if let Some(button) = button {
                unsafe { 
                    button.setImage(Some(&image));
                    button.setImagePosition(NSCellImagePosition::ImageOnly);
                }
                info!("Set button image");
            } else {
                info!("WARNING: Could not get button from status item!");
            }
        } else {
            info!("WARNING: Could not create system symbol image, falling back to text");
            let title = NSString::from_str("⌨️");
            let button = unsafe { status_item.button(mtm) };
            if let Some(button) = button {
                unsafe { button.setTitle(&title); }
                info!("Set button title to keyboard emoji");
            }
        }

        // Create the dropdown menu
        let menu = NSMenu::new(mtm);
        
        // Create "About" menu item
        let about_item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str("About TypoFixer"),
                Some(sel!(showAbout:)),
                &NSString::from_str(""),
            )
        };
        unsafe {
            about_item.setTarget(Some(self));
            menu.addItem(&about_item);
        }

        // Add separator
        menu.addItem(&NSMenuItem::separatorItem(mtm));

        // Create "Quit" menu item
        let quit_item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str("Quit TypoFixer"),
                Some(sel!(quitApp:)),
                &NSString::from_str("q"),
            )
        };
        unsafe {
            quit_item.setTarget(Some(self));
            menu.addItem(&quit_item);
        }

        // Attach the menu to the status item
        unsafe { status_item.setMenu(Some(&menu)); }

        // Prevent deallocation by leaking
        Box::leak(Box::new(status_item));
    }
}

// Public interface
pub struct MenuBar {
    delegate: Option<Retained<AppDelegate>>,
}

impl MenuBar {
    pub fn new() -> Self {
        Self { delegate: None }
    }

    pub fn setup(&mut self, mtm: MainThreadMarker) -> Result<(), Box<dyn std::error::Error>> {
        info!("Setting up menu-bar...");
        
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
        
        let delegate = AppDelegate::new(mtm);
        let delegate_ref = ProtocolObject::from_ref(&*delegate);
        app.setDelegate(Some(delegate_ref));
        
        unsafe { app.activate(); }
        
        self.delegate = Some(delegate);
        
        info!("✅ Menu-bar setup complete");
        Ok(())
    }

    pub fn run_event_loop(&self, mtm: MainThreadMarker) {
        NSApplication::sharedApplication(mtm).run()
    }
}

// ─────────────── Global helpers ──────────────────────
use std::sync::{Mutex, Once};

static INIT: Once = Once::new();
static mut MENU_BAR: Option<Mutex<MenuBar>> = None;

pub fn setup_menu_bar() -> Result<(), Box<dyn std::error::Error>> {
    INIT.call_once(|| {
        // SAFETY: This is safe because we're using Once to ensure this only runs once
        unsafe {
            MENU_BAR = Some(Mutex::new(MenuBar::new()));
        }
    });

    let mtm = MainThreadMarker::new().expect("must run on main thread");
    // SAFETY: Safe because we initialize it in call_once above
    #[allow(static_mut_refs)]
    let menu_bar = unsafe { MENU_BAR.as_ref().unwrap() };
    menu_bar.lock().unwrap().setup(mtm)
}

pub fn get_menu_bar() -> &'static Mutex<MenuBar> {
    // SAFETY: Safe because we initialize it in setup_menu_bar
    #[allow(static_mut_refs)]
    unsafe { MENU_BAR.as_ref().expect("menu bar not initialised") }
}

// ─────────────── About dialog ────────────────────────
unsafe fn show_about_dialog() {
    let mtm = MainThreadMarker::new().expect("must run on main thread");
    unsafe {
        let _pool = NSAutoreleasePool::new();
        let alert = NSAlert::new(mtm);
        alert.setMessageText(ns_string!("TypoFixer"));
        alert.setInformativeText(ns_string!(
            "A macOS spell-checking assistant that fixes typos in any text field.\n\n\
             Version 0.1.0\n\nPress ⌘⌥S to fix typos anywhere."
        ));
        alert.addButtonWithTitle(ns_string!("OK"));
        alert.runModal();
        info!("About dialog shown");
    }
}