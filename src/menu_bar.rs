use cocoa::appkit::{NSApp, NSApplicationActivationPolicyAccessory, NSStatusBar, NSMenu, NSMenuItem, NSVariableStatusItemLength};
use cocoa::base::{id, nil};
use cocoa::foundation::{NSString, NSAutoreleasePool};
use objc::{msg_send, sel, sel_impl, class, declare::ClassDecl, runtime::{Class, Object, Sel}};
use std::sync::{Arc, Mutex, Once};
use tracing::info;

// Create a delegate class for handling About action
static ABOUT_DELEGATE_CLASS: Once = Once::new();

unsafe fn create_about_delegate_class() -> *const Class {
    ABOUT_DELEGATE_CLASS.call_once(|| {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("AboutDelegate", superclass).unwrap();
        
        decl.add_method(sel!(aboutAction:), about_action_impl as extern "C" fn(&Object, Sel, id) -> ());
        
        decl.register();
    });
    
    class!(AboutDelegate)
}

extern "C" fn about_action_impl(_this: &Object, _cmd: Sel, _sender: id) {
    unsafe {
        show_about_dialog();
    }
}

// Thread-safe menu bar storage
pub struct MenuBar {
    status_item: Arc<Mutex<Option<id>>>,
    target: Arc<Mutex<Option<id>>>,
}

// Global menu bar instance
static mut MENU_BAR: Option<MenuBar> = None;
static MENU_BAR_INIT: Once = Once::new();

unsafe impl Send for MenuBar {}
unsafe impl Sync for MenuBar {}

impl MenuBar {
    pub fn new() -> Self {
        Self {
            status_item: Arc::new(Mutex::new(None)),
            target: Arc::new(Mutex::new(None)),
        }
    }

    pub fn setup(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Setting up menu bar icon...");
        
        unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            
            // Ensure NSApplication is properly initialized and finished launching
            let app = NSApp();
            
            // Set activation policy first
            let _: () = msg_send![app, setActivationPolicy: NSApplicationActivationPolicyAccessory];
            
            // Force the application to finish launching if it hasn't already
            let _: () = msg_send![app, finishLaunching];
            
            // Longer delay to ensure app is fully ready, especially for release builds
            std::thread::sleep(std::time::Duration::from_millis(300));
            
            // Get the system status bar
            let status_bar = NSStatusBar::systemStatusBar(nil);
            
            // Create a status item with variable length
            let status_item: id = msg_send![status_bar, statusItemWithLength: NSVariableStatusItemLength];
            
            // Set the title for the status item (using a keyboard emoji)
            let title = NSString::alloc(nil).init_str("⌨️");
            let _: () = msg_send![status_item, setTitle: title];
            
            // Create menu
            let menu: id = msg_send![NSMenu::alloc(nil), init];
            
            // Create the about delegate
            let about_delegate_class = create_about_delegate_class();
            let about_delegate: id = msg_send![about_delegate_class, alloc];
            let about_delegate: id = msg_send![about_delegate, init];
            
            // Add about menu item
            let about_title = NSString::alloc(nil).init_str("About TypoFixer");
            let about_item: id = msg_send![NSMenuItem::alloc(nil), 
                initWithTitle:about_title 
                action:sel!(aboutAction:) 
                keyEquivalent:NSString::alloc(nil).init_str("")
            ];
            
            // Set the target for the about action to our delegate
            let _: () = msg_send![about_item, setTarget: about_delegate];
            let _: () = msg_send![menu, addItem: about_item];
            
            // Store the delegate in the target field to keep it alive
            *self.target.lock().unwrap() = Some(about_delegate);
            
            // Add separator
            let separator = NSMenuItem::separatorItem(nil);
            let _: () = msg_send![menu, addItem: separator];
            
            // Add quit item
            let quit_title = NSString::alloc(nil).init_str("Quit TypoFixer");
            let quit_item: id = msg_send![NSMenuItem::alloc(nil), 
                initWithTitle:quit_title 
                action:sel!(terminate:) 
                keyEquivalent:NSString::alloc(nil).init_str("q")
            ];
            
            // Set the action target to NSApp
            let app = NSApp();
            let _: () = msg_send![quit_item, setTarget: app];
            let _: () = msg_send![menu, addItem: quit_item];
            
            // Set the menu to the status item
            let _: () = msg_send![status_item, setMenu: menu];
            
            // Store the status item
            *self.status_item.lock().unwrap() = Some(status_item);
            
            info!("✅ Menu bar icon created successfully");
        }
        
        Ok(())
    }

    pub fn run_event_loop(&self) {
        unsafe {
            let app = NSApp();
            let _: () = msg_send![app, run];
        }
    }
}

// Public API functions
pub fn setup_menu_bar() -> Result<(), Box<dyn std::error::Error>> {
    MENU_BAR_INIT.call_once(|| {
        unsafe {
            MENU_BAR = Some(MenuBar::new());
        }
    });
    
    let menu_bar = get_menu_bar()?;
    menu_bar.setup()
}

pub fn get_menu_bar() -> Result<&'static MenuBar, Box<dyn std::error::Error>> {
    unsafe {
        #[allow(static_mut_refs)]
        MENU_BAR.as_ref().ok_or("Menu bar not initialized".into())
    }
}

// Function to show the About dialog
unsafe fn show_about_dialog() {
    let _pool = NSAutoreleasePool::new(nil);
    
    // Create NSAlert
    let alert: id = msg_send![class!(NSAlert), alloc];
    let alert: id = msg_send![alert, init];
    
    // Set alert properties
    let message_text = NSString::alloc(nil).init_str("TypoFixer");
    let _: () = msg_send![alert, setMessageText: message_text];
    
    let info_text = NSString::alloc(nil).init_str("A macOS spell checking assistant that fixes typos in any text field.\n\nVersion 0.1.0\n\nPress ⌘⌥S to fix typos anywhere.");
    let _: () = msg_send![alert, setInformativeText: info_text];
    
    let button_text = NSString::alloc(nil).init_str("OK");
    let _: () = msg_send![alert, addButtonWithTitle: button_text];
    
    // Show the alert
    let _: () = msg_send![alert, runModal];
    
    info!("About dialog shown");
}