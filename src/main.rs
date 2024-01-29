use std::time::SystemTime;

use adw::gdk::{prelude::*, Key, ModifierType};
use adw::gio::Cancellable;
use adw::glib::Propagation;
use adw::gtk::EventControllerKey;
use adw::{Application, ApplicationWindow};
use webkit::{prelude::*, LoadEvent, WebInspector, WebView};

struct LastKey {
    key: Key,
    last_press_time: u64,
}

impl LastKey {
    fn new(key: Key, last_press_time: u64) -> Self {
        Self {
            key,
            last_press_time,
        }
    }

    fn is_composing(&self) -> bool {
        println!("last_press_time: {:?}", self.last_press_time);
        println!("get_current_time: {:?}", get_current_time());
        self.last_press_time + 500_000_000 > get_current_time()
    }
}

fn get_current_time() -> u64 {
    let time = SystemTime::now();

    time.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

static mut INSPECTOR: Option<WebInspector> = None;
static mut INSPECTOR_VISIBLE: bool = false;
static mut WEB_VIEW: Option<WebView> = None;
static mut LEADER_KEY: Option<LastKey> = None;

fn init_leader_key() {
    let key = Key::space;

    unsafe {
        LEADER_KEY = Some(LastKey::new(key, 0));
    };
}

fn leader_key() -> &'static LastKey {
    unsafe { LEADER_KEY.as_ref().unwrap() }
}

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
            INSPECTOR_VISIBLE = false;
        };
    });
}

fn init_webview() {
    unsafe { WEB_VIEW = Some(WebView::new()) };
}

fn show_key_press(key: Key, modifier_state: ModifierType, in_js_console: bool) {
    let mut res = String::new();

    if modifier_state.contains(ModifierType::SHIFT_MASK) {
        res.push_str("Shift+");
    }
    if modifier_state.contains(ModifierType::META_MASK) {
        // NOTE: Meta is almost never caught the webview or window
        res.push_str("Meta+");
    }
    if modifier_state.contains(ModifierType::CONTROL_MASK) {
        res.push_str("Control+");
    }
    if modifier_state.contains(ModifierType::ALT_MASK) {
        res.push_str("Alt+");
    }

    match key.to_unicode() {
        Some(chr) => res.push(chr),
        None => res.push_str(&format!("{:?}", key)),
    };

    if in_js_console {
        console_log(&res);
    } else {
        println!("{}", res);
    }
}

fn scrool_down_webview() {
    let webview = webview();
    let scroll_amount = 20;
    let javascript = format!("window.scrollBy(0, {})", scroll_amount);

    let cancellable: Option<&Cancellable> = None;

    webview.evaluate_javascript(javascript.as_str(), None, None, cancellable, |_| {});
}

fn scrool_up_webview() {
    let webview = webview();
    let scroll_amount = -20;
    let javascript = format!("window.scrollBy(0, {})", scroll_amount);

    let cancellable: Option<&Cancellable> = None;

    webview.evaluate_javascript(javascript.as_str(), None, None, cancellable, |_| {});
}

fn scrool_right_webview() {
    let webview = webview();
    let scroll_amount = 20;
    let javascript = format!("window.scrollBy({}, 0)", scroll_amount);

    let cancellable: Option<&Cancellable> = None;

    webview.evaluate_javascript(javascript.as_str(), None, None, cancellable, |_| {});
}

fn scrool_left_webview() {
    let webview = webview();
    let scroll_amount = -20;
    let javascript = format!("window.scrollBy({}, 0)", scroll_amount);

    let cancellable: Option<&Cancellable> = None;

    webview.evaluate_javascript(javascript.as_str(), None, None, cancellable, |_| {});
}

fn window_kb_input(
    event: &EventControllerKey,
    key: Key,
    keycode: u32,
    modifier_state: ModifierType,
) -> Propagation {
    _ = (event, key, keycode, modifier_state);

    print!("[window] ");
    show_key_press(key, modifier_state, true);
    show_key_press(key, modifier_state, false);

    if key == Key::h {
        scrool_left_webview();

        return Propagation::Stop;
    }
    if key == Key::j {
        scrool_down_webview();

        return Propagation::Stop;
    }
    if key == Key::k {
        scrool_up_webview();

        return Propagation::Stop;
    }
    if key == Key::l {
        scrool_right_webview();

        return Propagation::Stop;
    }

    let leader_key = leader_key();
    if key == leader_key.key {
        if leader_key.is_composing() {
            console_log("IS COMPSING A COMMAND");
        }

        unsafe {
            LEADER_KEY = Some(LastKey::new(key, get_current_time()));
        };

        return Propagation::Stop;
    }

    Propagation::Proceed
}

fn webkit_kb_input(
    event: &EventControllerKey,
    key: Key,
    keycode: u32,
    modifier_state: ModifierType,
) -> Propagation {
    _ = (event, keycode);
    print!("[web_view] ");
    show_key_press(key, modifier_state, true);
    show_key_press(key, modifier_state, false);

    if key == Key::h && modifier_state.contains(ModifierType::CONTROL_MASK) {
        scrool_left_webview();

        return Propagation::Stop;
    }
    if key == Key::j && modifier_state.contains(ModifierType::CONTROL_MASK) {
        scrool_down_webview();

        return Propagation::Stop;
    }
    if key == Key::k && modifier_state.contains(ModifierType::CONTROL_MASK) {
        scrool_up_webview();

        return Propagation::Stop;
    }
    if key == Key::l && modifier_state.contains(ModifierType::CONTROL_MASK) {
        scrool_right_webview();

        return Propagation::Stop;
    }

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

        if unsafe { INSPECTOR_VISIBLE } {
            inspector.close();
            unsafe { INSPECTOR_VISIBLE = false };
        } else {
            inspector.show();
            unsafe { INSPECTOR_VISIBLE = true };
        }

        // Prevents GTK inspector from showing up
        return Propagation::Stop;
    }

    if key == Key::semicolon && modifier_state.contains(ModifierType::CONTROL_MASK) {
        // Prevents smiley inputs from showing up
        return Propagation::Stop;
    }

    if key == Key::period && modifier_state.contains(ModifierType::CONTROL_MASK) {
        // Prevents smiley inputs from showing up
        return Propagation::Stop;
    }

    if key == Key::from_name("Escape").unwrap() && unsafe { INSPECTOR_VISIBLE } {
        inspector().close();
        unsafe { INSPECTOR_VISIBLE = false };
    }

    Propagation::Proceed
}

fn console_log(message: &str) {
    let webview = webview();

    let javascript = format!("console.log('{}')", message);
    let cancellable: Option<&Cancellable> = None;

    webview.evaluate_javascript(javascript.as_str(), None, None, cancellable, |_| {});
}

async fn in_insert_mode() -> bool {
    let webview = webview();

    let javascript = "document.activeElement.tagName === 'INPUT' || document.activeElement.tagName === 'TEXTAREA'";

    webview.evaluate_javascript_future(javascript, None, None).await.unwrap().to_boolean()
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
    init_leader_key();
    init_webview();
    let webview = webview();

    let web_view_key_pressed_controller = EventControllerKey::new();
    web_view_key_pressed_controller.connect_key_pressed(webkit_kb_input);
    webview.add_controller(web_view_key_pressed_controller);

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

    let window_key_pressed_controller = EventControllerKey::new();
    window_key_pressed_controller.connect_key_pressed(window_kb_input);
    window.add_controller(window_key_pressed_controller);

    window.present();
}

fn main() {
    let application = Application::builder()
        .application_id("com.igorgue.Browser")
        .build();

    application.connect_activate(activate);

    application.run();
}
