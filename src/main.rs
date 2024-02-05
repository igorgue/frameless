use std::cell::RefCell;
use std::include_str;
use std::rc::Rc;
use std::time::SystemTime;

use adw::gdk::{prelude::*, Key, ModifierType};
use adw::gio::Cancellable;
use adw::glib::Propagation;
use adw::gtk::EventControllerKey;
use adw::{Application, ApplicationWindow};
use webkit::{glib, javascriptcore, prelude::*, LoadEvent, WebInspector, WebView};

const LEADER_KEY_DEFAULT: Key = Key::semicolon;
const LEADER_KEY_COMPOSE_TIME: u64 = 500; // ms
const SCROLL_AMOUNT: i32 = 40;
const HOME_DEFAULT: &str = "https://crates.io";

struct Browser {
    home: String,
    inspector: WebInspector,
    inspector_visible: bool,
    web_view: WebView,
    leader_key: Rc<RefCell<LeaderKey>>,
    window: ApplicationWindow,
}

impl Browser {
    fn new(app: &Application) -> Self {
        let web_view = WebView::new();
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Browser")
            .default_width(350)
            .content(&web_view)
            .build();

        Self {
            home: std::env::var("BROWSER_HOME").unwrap_or(HOME_DEFAULT.to_string()),
            inspector: web_view.inspector().unwrap(),
            inspector_visible: false,
            web_view,
            leader_key: Rc::new(RefCell::new(LeaderKey::new(LEADER_KEY_DEFAULT, 0))),
            window,
        }
    }

    fn loaded(&self, webview: &WebView, event: LoadEvent) {
        _ = webview;

        if event != LoadEvent::Finished {
            return;
        }

        self.run_js(include_str!("vimium/lib/handler_stack.js"), |_| {});
        self.run_js(include_str!("vimium/lib/dom_utils.js"), |_| {});
        self.run_js(include_str!("vimium/lib/utils.js"), |_| {});
        self.run_js(include_str!("vimium/content_scripts/scroller.js"), |_| {});

        self.run_js("Scroller.init()", |_| {});

        self.console_log("Hello from Rust!");
    }

    fn run_js<F: Fn(Result<javascriptcore::Value, glib::Error>) + 'static>(
        &self,
        javascript: &str,
        f: F,
    ) {
        let c: Option<&Cancellable> = None;

        self.web_view
            .evaluate_javascript(javascript, None, None, c, f);
    }

    fn show(&self) {
        self.window.present();
    }

    fn quit(&self) {
        self.window.application().unwrap().quit();
    }

    fn close(&self) {
        self.window.close();
    }

    fn scroll_down(&self, times: u8) {
        let javascript = format!("Scroller.scrollBy('y', {} * {})", SCROLL_AMOUNT, times);
        self.run_js(javascript.as_str(), |_| {});
    }

    fn scroll_up(&self, times: u8) {
        let javascript = format!("Scroller.scrollBy('y', -1 * {} * {})", SCROLL_AMOUNT, times);
        self.run_js(javascript.as_str(), |_| {});
    }

    fn scroll_right(&self, times: u8) {
        let javascript = format!("Scroller.scrollBy('x', -1 * {} * {}", SCROLL_AMOUNT, times);
        self.run_js(javascript.as_str(), |_| {});
    }

    fn scroll_left(&self, times: u8) {
        let javascript = format!("Scroller.scrollBy('x', {} * {}", SCROLL_AMOUNT, times);
        self.run_js(javascript.as_str(), |_| {});
    }

    fn show_key_press(&self, key: Key, modifier_state: ModifierType, js_console: bool) {
        let mut res = String::new();

        if modifier_state.contains(ModifierType::SHIFT_MASK) {
            res.push_str("Shift+");
        }
        if modifier_state.contains(ModifierType::META_MASK) {
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

        if js_console {
            self.console_log(&res);
        } else {
            println!("{}", res);
        }
    }

    fn insert_mode<F: Fn(Result<javascriptcore::Value, glib::Error>) + 'static>(&self, f: F) {
        let javascript = "document.activeElement.tagName === 'INPUT' || document.activeElement.tagName === 'TEXTAREA'";
        self.run_js(javascript, f);
    }

    fn update_leader_key(&mut self, key: Key) {
        self.leader_key = Rc::new(RefCell::new(LeaderKey::new(key, get_current_time())));
    }

    fn window_kb_input(
        &mut self,
        event: &EventControllerKey,
        key: Key,
        keycode: u32,
        modifier_state: ModifierType,
    ) -> Propagation {
        _ = (event, keycode);

        print!("[window] ");
        self.show_key_press(key, modifier_state, false);

        // Movement
        if key == Key::h {
            self.scroll_left(1);

            return Propagation::Stop;
        }
        if key == Key::j {
            self.scroll_down(1);

            return Propagation::Stop;
        }
        if key == Key::k {
            self.scroll_up(1);

            return Propagation::Stop;
        }
        if key == Key::l {
            self.scroll_right(1);

            return Propagation::Stop;
        }

        // Back / Forward
        if key == Key::H {
            self.web_view.go_back();

            return Propagation::Stop;
        }
        if key == Key::L {
            self.web_view.go_forward();

            return Propagation::Stop;
        }

        // Leader key switches
        if key == LEADER_KEY_DEFAULT {
            self.update_leader_key(key);

            return Propagation::Stop;
        } else if self.leader_key.borrow_mut().is_composing() {
            if key == Key::q {
                println!("[browser] Quitting...");
                self.quit();

                return Propagation::Stop;
            }

            return Propagation::Stop;
        }

        Propagation::Proceed
    }

    fn webkit_kb_input(
        &mut self,
        event: &EventControllerKey,
        key: Key,
        keycode: u32,
        modifier_state: ModifierType,
    ) -> Propagation {
        _ = (event, keycode);
        print!("[web_view] ");
        self.show_key_press(key, modifier_state, false);

        // Scrool keys with h, j, k, l
        if key == Key::h && modifier_state.contains(ModifierType::CONTROL_MASK) {
            self.scroll_left(1);

            return Propagation::Stop;
        }
        if key == Key::j && modifier_state.contains(ModifierType::CONTROL_MASK) {
            self.scroll_down(1);

            return Propagation::Stop;
        }
        if key == Key::k && modifier_state.contains(ModifierType::CONTROL_MASK) {
            self.scroll_up(1);

            return Propagation::Stop;
        }
        if key == Key::l && modifier_state.contains(ModifierType::CONTROL_MASK) {
            self.scroll_right(1);

            return Propagation::Stop;
        }

        // Back / Forward
        if key == Key::H && modifier_state.contains(ModifierType::CONTROL_MASK) {
            self.web_view.go_back();

            return Propagation::Stop;
        }
        if key == Key::L && modifier_state.contains(ModifierType::CONTROL_MASK) {
            self.web_view.go_forward();

            return Propagation::Stop;
        }

        // Reload
        if key == Key::r && modifier_state.contains(ModifierType::CONTROL_MASK) {
            self.web_view.reload();
        }

        // Reload harder.
        if key == Key::R && modifier_state.contains(ModifierType::CONTROL_MASK) {
            self.web_view.reload_bypass_cache();
        }

        // Toggle inspector
        if key == Key::I && modifier_state.contains(ModifierType::CONTROL_MASK) {
            if self.inspector_visible {
                self.inspector.close();
                self.inspector_visible = false;
            } else {
                self.inspector.show();
                self.inspector_visible = true;
            }

            // Prevents GTK inspector from showing up
            return Propagation::Stop;
        }

        // Close window with Ctrl+w
        if key == Key::w && modifier_state.contains(ModifierType::CONTROL_MASK) {
            // TODO: Maybe close tab?
            self.close();

            return Propagation::Stop;
        }

        // Handle leader key
        if key == self.leader_key.borrow().key {
            let leader_key_clone = Rc::clone(&self.leader_key);

            // FIXME: Propagation::Stop is not returned here...
            // but should.
            self.insert_mode(move |res| {
                if let Ok(value) = res {
                    if value.to_boolean() {
                        if modifier_state.contains(ModifierType::CONTROL_MASK) {
                            *leader_key_clone.borrow_mut() =
                                LeaderKey::new(key, get_current_time());
                        }
                    } else {
                        *leader_key_clone.borrow_mut() = LeaderKey::new(key, get_current_time());
                    }
                }
            });
        } else if self.leader_key.borrow().is_composing() {
            if key == Key::q {
                println!("[browser] Quitting...");
                self.quit();

                return Propagation::Stop;
            }

            return Propagation::Stop;
        }

        // Remove features from GTK, smiles and add escape
        if key == Key::semicolon && modifier_state.contains(ModifierType::CONTROL_MASK) {
            // Prevents smiley inputs from showing up
            return Propagation::Stop;
        }

        if key == Key::period && modifier_state.contains(ModifierType::CONTROL_MASK) {
            // Prevents smiley inputs from showing up
            return Propagation::Stop;
        }

        if key == Key::from_name("Escape").unwrap() && self.inspector_visible {
            self.inspector.close();
            self.inspector_visible = false;
        }

        Propagation::Proceed
    }

    fn console_log(&self, message: &str) {
        let javascript = format!("console.log('{}')", message);
        self.run_js(javascript.as_str(), |_| {});
    }
}

struct LeaderKey {
    key: Key,
    last: u64,
}

impl LeaderKey {
    fn new(key: Key, last_press_time: u64) -> Self {
        Self {
            key,
            last: last_press_time,
        }
    }

    fn is_composing(&self) -> bool {
        self.last + LEADER_KEY_COMPOSE_TIME > get_current_time()
    }
}

fn get_current_time() -> u64 {
    let time = SystemTime::now();

    time.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn activate(app: &Application) {
    let browser = Rc::new(RefCell::new(Browser::new(app)));

    let window_key_pressed_controller = EventControllerKey::new();
    let browser_clone = Rc::clone(&browser);

    window_key_pressed_controller.connect_key_pressed(
        move |event, key, keycode, modifier_state| {
            browser_clone
                .borrow_mut()
                .window_kb_input(event, key, keycode, modifier_state)
        },
    );
    browser
        .borrow()
        .window
        .add_controller(window_key_pressed_controller);

    let web_view_key_pressed_controller = EventControllerKey::new();
    let browser_clone = Rc::clone(&browser);
    web_view_key_pressed_controller.connect_key_pressed(
        move |event, key, keycode, modifier_state| {
            browser_clone
                .borrow_mut()
                .webkit_kb_input(event, key, keycode, modifier_state)
        },
    );
    browser
        .borrow()
        .web_view
        .add_controller(web_view_key_pressed_controller);
    browser
        .borrow()
        .web_view
        .load_uri(browser.borrow().home.as_str());
    let browser_clone = Rc::clone(&browser);
    browser
        .borrow()
        .web_view
        .connect_load_changed(move |webview, event| {
            browser_clone.borrow_mut().loaded(webview, event);
        });

    let settings = WebViewExt::settings(&browser.borrow().web_view).unwrap();
    settings.set_enable_developer_extras(true);

    browser.borrow().window.present();

    let browser_clone = Rc::clone(&browser);
    browser.borrow().inspector.connect_closed(move |_| {
        browser_clone.borrow_mut().inspector_visible = false;
    });

    browser.borrow().show();
}

fn main() {
    let application = Application::builder()
        .application_id("com.igorgue.Browser")
        .build();

    application.connect_activate(activate);

    application.run();
}
