use adw::gdk::{Key, ModifierType};
use adw::gio::Cancellable;
use adw::glib::Propagation;
use adw::gtk::EventControllerKey;
use adw::{Application, ApplicationWindow};
use webkit::{prelude::*, LoadEvent, WebInspector, WebView};

static mut INSPECTOR: Option<WebInspector> = None;
static mut IS_INSPECTOR_VISIBLE: bool = false;

static mut WEB_VIEW: Option<WebView> = None;

fn inspector() -> &'static WebInspector {
    unsafe { INSPECTOR.as_ref().unwrap() }
}

fn webview() -> &'static WebView {
    unsafe { WEB_VIEW.as_ref().unwrap() }
}

fn init_inspector(webview: &WebView) {
    unsafe {
        INSPECTOR = Some(webview.inspector().unwrap());
    };

    inspector().connect_closed(|_| {
        unsafe {
            IS_INSPECTOR_VISIBLE = false;
        };
    });
}

fn init_webview() {
    unsafe { WEB_VIEW = Some(WebView::new()) };
}

fn show_key_press(key: Key, modifier_state: ModifierType) {
    if modifier_state.contains(ModifierType::CONTROL_MASK) {
        print!("Control+");
    }
    if modifier_state.contains(ModifierType::SHIFT_MASK) {
        print!("Shift+");
    }

    match key.to_unicode() {
        Some(chr) => println!("{}", chr),
        None => println!("{:?}", key),
    };
}

fn input(
    event: &EventControllerKey,
    key: Key,
    keycode: u32,
    modifier_state: ModifierType,
) -> Propagation {
    _ = (event, keycode);
    show_key_press(key, modifier_state);

    // Reload
    if key == Key::r && modifier_state.contains(ModifierType::CONTROL_MASK) {
        webview().reload();
    }

    // Reload harder.
    if key == Key::R && modifier_state.contains(ModifierType::CONTROL_MASK) {
        webview().reload_bypass_cache();
    }

    // Toggle inspector
    if key == Key::I && modifier_state.contains(ModifierType::CONTROL_MASK) {
        let inspector = inspector();

        if unsafe { IS_INSPECTOR_VISIBLE } {
            inspector.close();
            unsafe { IS_INSPECTOR_VISIBLE = false };
        } else {
            inspector.show();
            unsafe { IS_INSPECTOR_VISIBLE = true };
        }
    }

    Propagation::Stop
}

fn console_log(message: &str) {
    let webview = webview();

    let javascript = format!("console.log('{}')", message);
    let cancellable: Option<&Cancellable> = None;

    webview.evaluate_javascript(javascript.as_str(), None, None, cancellable, |_| {});
}

fn loaded(webview: &WebView, event: LoadEvent) {
    _ = webview;

    if event != LoadEvent::Finished {
        return;
    }

    println!("Loaded: {:?}", event);

    console_log("Hello from Rust!");
}

fn activate(app: &Application) {
    let key_pressed_controller = EventControllerKey::new();
    key_pressed_controller.connect_key_pressed(input);

    init_webview();
    let webview = webview();
    webview.load_uri("https://crates.io/");
    webview.connect_load_changed(loaded);

    let settings = WebViewExt::settings(webview).unwrap();
    settings.set_enable_developer_extras(true);

    init_inspector(webview);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Browser")
        .default_width(350)
        .content(webview)
        .build();

    window.add_controller(key_pressed_controller);

    window.present();
}

fn main() {
    let application = Application::builder()
        .application_id("com.igorgue.Browser")
        .build();

    application.connect_activate(activate);

    application.run();
}
