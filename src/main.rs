use std::cell::RefCell;
use std::time::SystemTime;

use adw::gdk::{Key, ModifierType};
use adw::gio::Cancellable;
use adw::gtk::EventControllerKey;
use adw::prelude::*;
use adw::{glib, glib::Propagation};
use adw::{Application, ApplicationWindow};

use webkit::prelude::*;
// use webkit::{javascriptcore, LoadEvent, WebView};
use webkit::{LoadEvent, WebView};

const LEADER_KEY_DEFAULT: Key = Key::semicolon;
const LEADER_KEY_COMPOSE_TIME: u64 = 500; // ms
const DEFAULT_WINDOW_WIDTH: i32 = 300;
// const SCROLL_AMOUNT: i32 = 40;
const HOME_DEFAULT: &str = "https://crates.io";

#[derive(Debug, Clone)]
struct LeaderKey {
    key: Key,
    last: u64,
}

impl LeaderKey {
    fn new(key: Key, last: u64) -> Self {
        Self { key, last }
    }

    fn is_composing(&self) -> bool {
        self.last + LEADER_KEY_COMPOSE_TIME > get_current_time()
    }

    fn update(&mut self) {
        self.last = get_current_time();
    }
}

fn build_ui(app: &Application) {
    let webviews: Vec<WebView> = vec![];

    let tab_bar = adw::TabBar::builder().build();
    let tab_view = adw::TabView::builder().build();

    let leader_key = LeaderKey::new(LEADER_KEY_DEFAULT, 0);

    tab_bar.set_view(Some(&tab_view));

    let toolbar_view = adw::ToolbarView::new();

    toolbar_view.add_top_bar(&tab_bar);
    toolbar_view.set_content(Some(&tab_view));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Frameless")
        .default_width(DEFAULT_WINDOW_WIDTH)
        .content(&toolbar_view)
        .build();

    let window_key_pressed_controller = EventControllerKey::new();
    let leader_key_ref = RefCell::new(leader_key);
    let window_clone = window.clone();
    let webviews_ref = RefCell::new(webviews.clone());
    window_key_pressed_controller.connect_key_pressed(
        move |event, key, keycode, modifier_state| {
            _ = (event, keycode);

            print!("[kbd event] ");
            show_key_press(key, modifier_state);

            if key == leader_key_ref.borrow().key {
                leader_key_ref.borrow_mut().update();
                return Propagation::Stop;
            }

            if leader_key_ref.borrow().is_composing() {
                if key == Key::q {
                    println!("[frameless] Quitting!");

                    window_clone.application().unwrap().quit();
                    return Propagation::Stop;
                }

                if key == Key::n {
                    println!("[frameless] New tab!");

                    let url = HOME_DEFAULT;
                    let webview = WebView::new();

                    let settings = WebViewExt::settings(&webview).unwrap();
                    settings.set_enable_developer_extras(true);

                    webview.load_uri(url);
                    webviews_ref.borrow_mut().push(webview);

                    let tab_view = tab_bar.view().unwrap();

                    let index = webviews_ref.borrow().len() - 1;

                    println!("Index: {}", index);
                    tab_view.append(&webviews_ref.borrow()[index]);

                    let tab_page = tab_view.page(&webviews_ref.borrow()[index]);
                    tab_view.set_selected_page(&tab_page);

                    let tab_page_clone = tab_page.clone();
                    let window_clone = window_clone.clone();
                    let webview_clone = webviews_ref.borrow()[index].clone();
                    webview_clone.connect_load_changed(move |webview, event| {
                        tab_page_clone.set_title("New tab");

                        if event == LoadEvent::Finished {
                            let c: Option<&Cancellable> = None;

                            let window_clone = window_clone.clone();
                            let tab_page_clone = window_clone.clone();

                            webview.evaluate_javascript(
                                "document.title",
                                None,
                                None,
                                c,
                                move |res| {
                                    if let Ok(value) = res {
                                        let title = value.to_string();
                                        tab_page_clone.set_title(Some(title.as_str()));
                                        window_clone.set_title(Some(title.as_str()));
                                    }
                                },
                            );
                            webview.evaluate_javascript(
                                include_str!("vimium/lib/handler_stack.js"),
                                None,
                                None,
                                c,
                                |_| {},
                            );
                            webview.evaluate_javascript(
                                include_str!("vimium/lib/dom_utils.js"),
                                None,
                                None,
                                c,
                                |_| {},
                            );
                            webview.evaluate_javascript(
                                include_str!("vimium/lib/utils.js"),
                                None,
                                None,
                                c,
                                |_| {},
                            );
                            webview.evaluate_javascript(
                                include_str!("vimium/content_scripts/scroller.js"),
                                None,
                                None,
                                c,
                                |_| {},
                            );
                            webview.evaluate_javascript("Scroller.init()", None, None, c, |_| {});

                            // TODO: we gonna do something here... maybe listen to all keyboard
                            // events
                            let webview_key_pressed_controller = EventControllerKey::new();
                            let webview_clone_clone = webview.clone();
                            webview_key_pressed_controller.connect_key_pressed(move |event, key, keycode, modifier_state| {
                                _ = (event, keycode);

                                print!("[kbd event] ");
                                show_key_press(key, modifier_state);

                                let js = "document.activeElement.tagName === 'INPUT' || document.activeElement.tagName === 'TEXTAREA'";
                                webview_clone_clone.evaluate_javascript(js, None, None, c, move |res| {
                                    if let Ok(value) = res {
                                        if value.to_boolean() {
                                            println!("YES insert mode, val: {}", value);
                                        } else {
                                            println!("NO insert mode, val: {}", value);
                                        }
                                    }
                                });

                                Propagation::Proceed
                            });

                            webview.add_controller(webview_key_pressed_controller);
                        }
                    });

                    webviews_ref.borrow()[index].grab_focus();

                    return Propagation::Stop;
                }

                return Propagation::Stop;
            }

            Propagation::Proceed
        },
    );

    window.add_controller(window_key_pressed_controller);
    window.show();
}

// fn run_js<F: Fn(Result<javascriptcore::Value, glib::Error>) + 'static>(
//     web_view: &WebView,
//     javascript: &str,
//     f: F,
// ) {
//     let c: Option<&Cancellable> = None;
//
//     web_view.evaluate_javascript(javascript, None, None, c, f);
// }

fn show_key_press(key: Key, modifier_state: ModifierType) {
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

    println!("{}", res);
}

fn get_current_time() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn main() -> glib::ExitCode {
    let application = Application::builder()
        .application_id("com.igorgue.Frameless")
        .build();

    application.connect_activate(build_ui);
    application.run()
}
