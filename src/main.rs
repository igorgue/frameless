use std::env;
use std::time::SystemTime;
use std::{cell::RefCell, rc::Rc};

use adw::gdk::{Key, ModifierType};
use adw::gio::Cancellable;
use adw::gtk::EventControllerKey;
use adw::prelude::*;
use adw::{glib, glib::clone, glib::Propagation};
use adw::{Application, ApplicationWindow};

use webkit::prelude::*;
use webkit::{javascriptcore, LoadEvent, WebView};

const APP_ID: &str = "com.igorgue.Frameless";
const LEADER_KEY_DEFAULT: Key = Key::semicolon;
const LEADER_KEY_COMPOSE_TIME: u64 = 500; // ms
const DEFAULT_WINDOW_WIDTH: i32 = 300;
const SCROLL_AMOUNT: i32 = 22;
const HOME_DEFAULT: &str = "https://crates.io";

static mut LEADER_LAST_PRESSED: u64 = 0;

fn leader_last_pressed() -> u64 {
    unsafe { LEADER_LAST_PRESSED }
}

fn set_leader_last_pressed(time: u64) {
    unsafe {
        LEADER_LAST_PRESSED = time;
    }
}

fn update_leader_last_pressed() {
    set_leader_last_pressed(get_current_time());
}

fn leader_is_composing() -> bool {
    let last_pressed = leader_last_pressed();
    let current_time = get_current_time();

    current_time - last_pressed < LEADER_KEY_COMPOSE_TIME
}

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

fn run_js<F: Fn(Result<javascriptcore::Value, glib::Error>) + 'static>(
    web_view: &WebView,
    javascript: &str,
    f: F,
) {
    let c: Option<&Cancellable> = None;

    web_view.evaluate_javascript(javascript, None, None, c, f);
}

fn scroll_up(web_view: &WebView, times: u8) {
    let js = format!("Scroller.scrollBy('y', -{} * {})", SCROLL_AMOUNT, times);
    run_js(web_view, js.as_str(), |_| {});
}

fn scroll_down(web_view: &WebView, times: u8) {
    let js = format!("Scroller.scrollBy('y', {} * {})", SCROLL_AMOUNT, times);

    run_js(web_view, js.as_str(), |_| {});
}

fn scroll_left(web_view: &WebView, times: u8) {
    let js = format!("Scroller.scrollBy('x', -{} * {})", SCROLL_AMOUNT, times);
    run_js(web_view, js.as_str(), |_| {});
}

fn scroll_right(web_view: &WebView, times: u8) {
    let js = format!("Scroller.scrollBy('x', {} * {})", SCROLL_AMOUNT, times);
    run_js(web_view, js.as_str(), |_| {});
}

fn init_settings(web_view: &WebView) {
    let settings = WebViewExt::settings(web_view).expect("No settings found");

    settings.set_enable_developer_extras(true);
    settings.set_enable_caret_browsing(false);
    settings.set_enable_smooth_scrolling(true);
    settings.set_enable_back_forward_navigation_gestures(true);
    settings.set_enable_webgl(true);
    settings.set_enable_webaudio(true);
    settings.set_javascript_can_open_windows_automatically(true);
    settings.set_allow_modal_dialogs(true);
}

fn get_current_time() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn new_tab(
    window: &ApplicationWindow,
    webviews: &mut Vec<WebView>,
    tab_bar: &adw::TabBar,
    developer_extras: bool,
) {
    let url = HOME_DEFAULT;
    let webview = WebView::new();

    init_settings(&webview);

    webview.load_uri(url);
    webviews.push(webview);

    let tab_view = tab_bar.view().unwrap();

    let index = webviews.len() - 1;

    tab_view.append(&webviews[index]);

    let tab_page = tab_view.page(&webviews[index]);
    tab_view.set_selected_page(&tab_page);

    let webview = webviews[index].clone();
    webview.connect_load_changed(
        clone!(@strong window, @strong tab_page => move |webview, event| {
            handle_webkit_load_changed(webview, event, &window, &tab_page);
        }),
    );

    let webview_key_pressed_controller = EventControllerKey::new();
    let webviews_ref = Rc::new(RefCell::new(webviews.clone()));
    webview_key_pressed_controller.connect_key_pressed(
        clone!(@strong window, @strong webview, @strong tab_bar, @strong webviews => move |_event, key, _keycode, modifier_state| {
            handle_webkit_key_press(&window, &tab_bar, webviews_ref.borrow_mut().as_mut(), key, modifier_state, developer_extras)
        })
    );

    webviews[index].add_controller(webview_key_pressed_controller);
    webviews[index].grab_focus();
}

fn handle_window_key_press(
    window: &ApplicationWindow,
    tab_bar: &adw::TabBar,
    webviews: &mut Vec<WebView>,
    key: Key,
    modifier_state: ModifierType,
) -> Propagation {
    print!("[kbd event] ");

    let developer_extras = true;

    show_key_press(key, modifier_state);

    if key == LEADER_KEY_DEFAULT {
        println!("[frameless] Leader key pressed!");
        update_leader_last_pressed();
        return Propagation::Stop;
    }

    if leader_is_composing() {
        if key == Key::q {
            println!("[frameless] Quitting!");

            window.application().unwrap().quit();
            return Propagation::Stop;
        }

        if key == Key::n {
            println!("[frameless] New tab!");

            new_tab(window, webviews, tab_bar, developer_extras);

            return Propagation::Stop;
        }

        return Propagation::Stop;
    }

    Propagation::Proceed
}

fn handle_webkit_load_changed(
    webview: &WebView,
    event: LoadEvent,
    window: &ApplicationWindow,
    tab_page: &adw::TabPage,
) {
    tab_page.set_title("New Tab");

    if event != LoadEvent::Finished {
        return;
    }

    let c: Option<&Cancellable> = None;

    webview.evaluate_javascript(
        "document.title",
        None,
        None,
        c,
        clone!(@strong window, @strong tab_page => move |res| {
            if let Ok(value) = res {
                let title = value.to_string();
                tab_page.set_title(title.as_str());
                window.set_title(Some(title.as_str()));

                println!("Title: {}", title);
            }
        }),
    );

    let c: Option<&Cancellable> = None;

    webview.evaluate_javascript(
        "if (!window.fml) window.fml = { loaded: false }",
        None,
        None,
        c,
        |_| {},
    );

    webview.evaluate_javascript(
        "fml.loaded === false",
        None,
        None,
        c,
        clone!(@strong webview => move |res| {
            if let Ok(value) = res {
                if value.to_boolean() {
                    println!("[frameless] loading vimium...");

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
                    webview.evaluate_javascript(
                        "Scroller.init()",
                        None,
                        None,
                        c,
                        |_| {},
                    );

                    webview.evaluate_javascript(
                        "fml.loaded = true",
                        None,
                        None,
                        c,
                        |_| {},
                    );
                }
            }
        }),
    );
}

#[allow(clippy::ptr_arg)]
fn handle_webkit_key_press(
    window: &ApplicationWindow,
    tab_bar: &adw::TabBar,
    webviews: &mut Vec<WebView>,
    key: Key,
    modifier_state: ModifierType,
    developer_extras: bool,
) -> Propagation {
    print!("[kbd event] ");
    show_key_press(key, modifier_state);

    let webview = webviews.last().unwrap();

    // Check if the active element is an input or textarea
    // similar to vim insert mode / normal mode distinction
    // insert mode should allow all typing keys to work
    // normal mode should allow all vim keys to work
    let js = "document.activeElement.tagName === 'INPUT' || document.activeElement.tagName === 'TEXTAREA'";
    let c: Option<&Cancellable> = None;
    let webviews_ref = Rc::new(RefCell::new(webviews.clone()));
    webview.evaluate_javascript(js, None, None, c,
        clone!(@strong window, @strong webview, @strong webviews, @strong tab_bar => move |res| {
            if let Ok(value) = res {
                // insert mode
                if value.to_boolean() {
                    // ctrl + leader key
                    if key == LEADER_KEY_DEFAULT && modifier_state.contains(ModifierType::CONTROL_MASK) {
                        update_leader_last_pressed();
                    }

                    if leader_is_composing() {
                        if key == Key::q {
                            println!("[frameless] Quitting!");

                            window.application().unwrap().quit();
                        }

                        if key == Key::n {
                            println!("[frameless] New tab!");

                            new_tab(&window, webviews_ref.borrow_mut().as_mut(), &tab_bar, developer_extras);
                        }
                    }

                    // Scrool keys with ctrl + h, j, k, l
                    if key == Key::h && modifier_state.contains(ModifierType::CONTROL_MASK) {
                        scroll_left(&webview, 1);
                    }
                    if key == Key::j && modifier_state.contains(ModifierType::CONTROL_MASK) {
                        scroll_down(&webview, 1);
                    }
                    if key == Key::k && modifier_state.contains(ModifierType::CONTROL_MASK) {
                        scroll_up(&webview, 1);
                    }
                    if key == Key::l && modifier_state.contains(ModifierType::CONTROL_MASK) {
                        scroll_right(&webview, 1);
                    }

                    // Back / Forward with ctrl + h, l
                    if key == Key::H && modifier_state.contains(ModifierType::CONTROL_MASK) {
                        webview.go_back();
                    }
                    if key == Key::L && modifier_state.contains(ModifierType::CONTROL_MASK) {
                        webview.go_forward();
                    }
                // normal mode
                } else {
                    // leader key
                    if key == LEADER_KEY_DEFAULT {
                        update_leader_last_pressed();
                    }

                    if leader_is_composing() {
                        if key == Key::q {
                            println!("[frameless] Quitting!");

                            window.application().unwrap().quit();
                        }

                        if key == Key::n {
                            println!("[frameless] New tab!");

                            new_tab(&window, webviews_ref.borrow_mut().as_mut(), &tab_bar, developer_extras);
                        }
                    } else {

                        // Scrool keys with h, j, k, l
                        if key == Key::h {
                            scroll_left(&webview, 1);
                        }
                        if key == Key::j {
                            scroll_down(&webview, 1);
                        }
                        if key == Key::k {
                            scroll_up(&webview, 1);
                        }
                        if key == Key::l {
                            scroll_right(&webview, 1);
                        }

                        // Back / Forward with H, L
                        if key == Key::H {
                            webview.go_back();
                        }
                        if key == Key::L {
                            webview.go_forward();
                        }
                        if key == Key::r {
                            webview.reload();
                        }
                    }
                }

                // these keys work for both modes

                // Reload with ctrl + r / reload harder with ctrl + R
                if key == Key::r && modifier_state.contains(ModifierType::CONTROL_MASK) {
                    webview.reload();
                }
                if key == Key::R && modifier_state.contains(ModifierType::CONTROL_MASK) {
                    webview.reload_bypass_cache();
                }

                // Toggle inspector with ctrl + I
                if developer_extras && key == Key::I && modifier_state.contains(ModifierType::CONTROL_MASK) {
                    let inspector = webview.inspector().unwrap();

                    if inspector.is_attached() {
                        inspector.close();
                    } else {
                        inspector.show();
                    }
                }

                // Close inspector with escape
                if developer_extras && key == Key::Escape {
                    let inspector = webview.inspector().unwrap();

                    if inspector.is_attached() {
                        inspector.close();
                    }
                }
            }
        })
    );

    // Remove features from GTK, smiles menu
    if (key == Key::semicolon || key == Key::period)
        && modifier_state.contains(ModifierType::CONTROL_MASK)
    {
        return Propagation::Stop;
    }

    Propagation::Proceed
}

fn build_ui(app: &Application) {
    let webviews: Vec<WebView> = vec![];

    let tab_bar = adw::TabBar::builder().build();
    let tab_view = adw::TabView::builder().build();

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
    let webviews_ref = Rc::new(RefCell::new(webviews));
    let webviews_ref2 = webviews_ref.clone();
    let tab_bar_clone = tab_bar.clone();
    window_key_pressed_controller.connect_key_pressed(
        clone!(@strong window => move |_event, key, _keycode, modifier_state| {
            handle_window_key_press(
                &window,
                &tab_bar,
                &mut webviews_ref.borrow_mut(),
                key,
                modifier_state,
            )
        }),
    );
    window.add_controller(window_key_pressed_controller);

    new_tab(
        &window,
        &mut webviews_ref2.borrow_mut(),
        &tab_bar_clone,
        true,
    );

    window.show();

    // TODO: add homepage...
    // let content = "<html><body><h1>Frameless</h1><p>Press <code>;</code> to start typing commands</p></body></html>";
    // let webview = WebView::new();
    //
    // init_settings(&webview);
    // webview.load_html(content, None);

    // tab_view.append(&webview);
}

fn main() -> glib::ExitCode {
    // NOTE: enable when the wayland compositing stuff is resolved
    if env::var("WAYLAND_DISPLAY").is_ok() {
        env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    let application = Application::builder().application_id(APP_ID).build();

    application.connect_activate(build_ui);
    application.run()
}
