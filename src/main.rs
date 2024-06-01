use core::borrow::{Borrow, BorrowMut};
use std::rc::Rc;
use std::time::SystemTime;
use std::{cell::RefCell, sync::Mutex};

use adw::gdk::{Key, ModifierType};
use adw::gio::Cancellable;
use adw::glib::Propagation;
use adw::gtk::EventControllerKey;
use adw::prelude::*;
use adw::{Application, ApplicationWindow};
use webkit::{glib, javascriptcore, prelude::*, LoadEvent, WebView};

const LEADER_KEY_DEFAULT: Key = Key::semicolon;
const LEADER_KEY_COMPOSE_TIME: u64 = 500; // ms
const DEFAULT_WINDOW_WIDTH: i32 = 300;
const SCROLL_AMOUNT: i32 = 40;
const HOME_DEFAULT: &str = "https://crates.io";

#[derive(Debug, Clone)]
struct Page {
    web_view: WebView,
    browser: Rc<RefCell<Browser>>,
    inspector_visible: bool,
}

impl Page {
    fn new(developer: bool, browser: Rc<RefCell<Browser>>) -> Rc<RefCell<Self>> {
        let web_view = WebView::new();
        let inspector_visible = false;

        if developer {
            let settings = WebViewExt::settings(&web_view).unwrap();
            settings.set_enable_developer_extras(developer);
        }

        let page = Rc::new(RefCell::new(Self {
            web_view,
            browser,
            inspector_visible,
        }));

        // Uncomment the following lines if you want to re-enable the functionality
        // let page_clone = Rc::clone(&page);
        // page.borrow().web_view.connect_load_changed(move |web_view, event| {
        //     Page::loaded(&page_clone, web_view, event);
        // });

        page
    }

    fn load_url(&self, url: &str) {
        self.web_view.load_uri(url);
    }

    fn toggle_inspector(&mut self) {
        if self.inspector_visible {
            self.web_view.inspector().unwrap().close();
        } else {
            self.web_view.inspector().unwrap().show();
        }

        self.update_inspector_state(!self.inspector_visible);
    }

    fn update_inspector_state(&mut self, is_visible: bool) {
        self.inspector_visible = is_visible;
    }

    fn loaded(page: &Rc<RefCell<Self>>, web_view: &WebView, event: LoadEvent) {
        if event != LoadEvent::Finished {
            return;
        }

        Page::run_js(
            web_view,
            include_str!("vimium/lib/handler_stack.js"),
            |_| {},
        );
        Page::run_js(web_view, include_str!("vimium/lib/dom_utils.js"), |_| {});
        Page::run_js(web_view, include_str!("vimium/lib/utils.js"), |_| {});
        Page::run_js(
            web_view,
            include_str!("vimium/content_scripts/scroller.js"),
            |_| {},
        );
        Page::run_js(web_view, "Scroller.init()", |_| {});
    }

    fn run_js<F: Fn(Result<javascriptcore::Value, glib::Error>) + 'static>(
        web_view: &WebView,
        javascript: &str,
        f: F,
    ) {
        let c: Option<&Cancellable> = None;
        web_view.evaluate_javascript(javascript, None, None, c, f);
    }

    fn console_log(&self, message: &str) {
        let javascript = format!("console.log('{}')", message);
        Page::run_js(&self.web_view, &javascript, |_| {});
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
        }

        println!("{}", res);
    }

    fn scroll_down(&self, times: u8) {
        let javascript = format!("Scroller.scrollBy('y', {} * {})", SCROLL_AMOUNT, times);
        Page::run_js(&self.web_view, &javascript, |_| {});
    }

    fn scroll_up(&self, times: u8) {
        let javascript = format!("Scroller.scrollBy('y', -1 * {} * {})", SCROLL_AMOUNT, times);
        Page::run_js(&self.web_view, &javascript, |_| {});
    }

    fn scroll_right(&self, times: u8) {
        let javascript = format!("Scroller.scrollBy('x', -1 * {} * {})", SCROLL_AMOUNT, times);
        Page::run_js(&self.web_view, &javascript, |_| {});
    }

    fn scroll_left(&self, times: u8) {
        let javascript = format!("Scroller.scrollBy('x', {} * {})", SCROLL_AMOUNT, times);
        Page::run_js(&self.web_view, &javascript, |_| {});
    }

    fn insert_mode<F: Fn(Result<javascriptcore::Value, glib::Error>) + 'static>(&self, f: F) {
        let javascript =
            "document.activeElement.tagName === 'INPUT' || document.activeElement.tagName === 'TEXTAREA'";
        Page::run_js(&self.web_view, javascript, f);
    }

    fn webkit_kb_input(
        page: Rc<RefCell<Self>>,
        event: &EventControllerKey,
        key: Key,
        keycode: u32,
        modifier_state: ModifierType,
    ) -> Propagation {
        _ = (event, keycode);

        // page.borrow().show_key_press(key, modifier_state, true);
        //
        // let page_clone = Rc::clone(&page);
        // page.borrow().insert_mode(move |res| {
        //     let page_clone = Rc::clone(&page_clone);
        //     if let Ok(value) = res {
        //         if value.to_boolean() {
        //             if key == Key::h && modifier_state.contains(ModifierType::CONTROL_MASK) {
        //                 page_clone.borrow().scroll_left(1);
        //             }
        //             if key == Key::j && modifier_state.contains(ModifierType::CONTROL_MASK) {
        //                 page_clone.borrow().scroll_down(1);
        //             }
        //             if key == Key::k && modifier_state.contains(ModifierType::CONTROL_MASK) {
        //                 page_clone.borrow().scroll_up(1);
        //             }
        //             if key == Key::l && modifier_state.contains(ModifierType::CONTROL_MASK) {
        //                 page_clone.borrow().scroll_right(1);
        //             }
        //             if key == Key::H && modifier_state.contains(ModifierType::CONTROL_MASK) {
        //                 page_clone.borrow().web_view.go_back();
        //             }
        //             if key == Key::L && modifier_state.contains(ModifierType::CONTROL_MASK) {
        //                 page_clone.borrow().web_view.go_forward();
        //             }
        //         } else {
        //             if key == Key::h {
        //                 page_clone.borrow().scroll_left(1);
        //             }
        //             if key == Key::j {
        //                 page_clone.borrow().scroll_down(1);
        //             }
        //             if key == Key::k {
        //                 page_clone.borrow().scroll_up(1);
        //             }
        //             if key == Key::l {
        //                 page_clone.borrow().scroll_right(1);
        //             }
        //             if key == Key::H {
        //                 page_clone.borrow().web_view.go_back();
        //             }
        //             if key == Key::L {
        //                 page_clone.borrow().web_view.go_forward();
        //             }
        //         }
        //     }
        // });
        //
        // if key == Key::r && modifier_state.contains(ModifierType::CONTROL_MASK) {
        //     page.borrow().web_view.reload();
        //     return Propagation::Stop;
        // }
        //
        // if key == Key::R && modifier_state.contains(ModifierType::CONTROL_MASK) {
        //     page.borrow().web_view.reload_bypass_cache();
        //     return Propagation::Stop;
        // }
        //
        // if key == Key::I && modifier_state.contains(ModifierType::CONTROL_MASK) {
        //     page.borrow_mut().toggle_inspector();
        //     return Propagation::Stop;
        // }
        //
        // if key == Key::w && modifier_state.contains(ModifierType::CONTROL_MASK) {
        //     page.borrow().web_view.try_close();
        //     return Propagation::Stop;
        // }
        //
        // if key == page.borrow().browser.borrow().leader_key.borrow().key {
        //     let leader_key_clone = Rc::clone(&page.borrow().browser.borrow().leader_key);
        //
        //     page.borrow().insert_mode(move |res| {
        //         if let Ok(value) = res {
        //             if value.to_boolean() {
        //                 if modifier_state.contains(ModifierType::CONTROL_MASK) {
        //                     *leader_key_clone.borrow_mut() =
        //                         LeaderKey::new(key, get_current_time());
        //                 }
        //             } else {
        //                 *leader_key_clone.borrow_mut() = LeaderKey::new(key, get_current_time());
        //             }
        //         }
        //     });
        // } else if page.borrow().browser.borrow().leader_key.borrow().is_composing() {
        //     if key == Key::q {
        //         println!("[browser] Quitting...");
        //         page.borrow().browser.borrow().window.application().unwrap().quit();
        //         return Propagation::Stop;
        //     }
        //
        //     return Propagation::Stop;
        // }

        Propagation::Proceed
    }
}

#[derive(Debug, Clone)]
struct Browser {
    leader_key: LeaderKey,
    window: ApplicationWindow,
    tab_bar: adw::TabBar,
    pages: Vec<Rc<RefCell<Page>>>,
}

static mut BROWSER: Mutex<Option<Browser>> = Mutex::new(None);

impl Browser {
    fn new(app: &Application) -> Self {
        let tab_bar = adw::TabBar::builder().build();
        let tab_view = adw::TabView::builder().build();

        tab_bar.set_view(Some(&tab_view));

        let toolbar_view = adw::ToolbarView::new();

        toolbar_view.add_top_bar(&tab_bar);
        toolbar_view.set_content(Some(&tab_view));

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Browser")
            .default_width(350)
            .content(&toolbar_view)
            .build();

        let browser = Self {
            leader_key: LeaderKey::new(LEADER_KEY_DEFAULT, 0),
            window,
            tab_bar,
            pages: vec![],
        };

        let window_key_pressed_controller = EventControllerKey::new();
        let browser_clone = browser.clone();
        window_key_pressed_controller.connect_key_pressed(
            move |event, key, keycode, modifier_state| {
                let mut browser_clone = browser_clone.clone();
                browser_clone
                    .borrow_mut()
                    .window_kb_input(event, key, keycode, modifier_state);

                Propagation::Proceed
            },
        );
        let browser_clone = browser.clone();
        browser_clone
            .window
            .add_controller(window_key_pressed_controller);

        browser
    }

    fn show(&mut self) {
        self.window.show();
    }

    fn quit(&self) {
        self.window.application().unwrap().quit();
    }

    fn update_leader_key(&mut self, key: Key) {
        self.leader_key = LeaderKey::new(key, get_current_time());
    }

    fn show_key_press(&self, key: Key, modifier_state: ModifierType) {
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

    fn new_tab(self) {
        let url = HOME_DEFAULT;
        let developer = true;

        let browser_clone = Rc::new(RefCell::new(self));
        let page = Page::new(developer, browser_clone);
        // page.borrow().load_url(url);
        //
        // self.pages.push(page.clone());
        //
        // let tab_view = self.tab_bar.view().unwrap();
        //
        // let index = self.pages.len() - 1;
        // tab_view.append(&self.pages[index].borrow().web_view);
        //
        // let tab_page = tab_view.page(&self.pages[index].borrow().web_view);
        // tab_view.set_selected_page(&tab_page);
        //
        // let page_clone = Rc::clone(&self.pages[index]);
        // let tab_page_clone = tab_page.clone();
        // self.pages[index].borrow().web_view.connect_load_changed(move |_, _| {
        //     let tab_page_clone = tab_page_clone.clone();
        //
        //     Page::run_js(&page_clone.borrow().web_view, "document.title", move |res| {
        //         if let Ok(value) = res {
        //             let title = value.to_string();
        //             tab_page_clone.set_title(title.as_str());
        //         }
        //     });
        // });
        //
        // self.pages[index].borrow().web_view.grab_focus();
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
        self.show_key_press(key, modifier_state);

        if key == self.leader_key.key {
            self.update_leader_key(key);

            println!("Leader key pressed! At: {:?}", self.leader_key.last);
            return Propagation::Stop;
        }

        if self.leader_key.is_composing() {
            println!("[browser] compossing...");
            if key == Key::q {
                println!("[browser] Quitting...");
                self.quit();
                return Propagation::Stop;
            }

            if key == Key::n {
                println!("[browser] New tab...");
                self.clone().new_tab();
                return Propagation::Stop;
            }

            return Propagation::Stop;
        }

        Propagation::Proceed
    }
}

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
        .title("Browser")
        .default_width(DEFAULT_WINDOW_WIDTH)
        .content(&toolbar_view)
        .build();

    let window_key_pressed_controller = EventControllerKey::new();
    let leader_key_ref = RefCell::new(leader_key);
    let window_ref = RefCell::new(window.clone());
    let webviews_ref = RefCell::new(webviews.clone());
    window_key_pressed_controller.connect_key_pressed(
        move |event, key, keycode, modifier_state| {
            _ = (event, keycode);

            print!("[window kbd event] ");
            show_key_press(key, modifier_state);

            if key == leader_key_ref.borrow().key {
                leader_key_ref.borrow_mut().update();
                return Propagation::Stop;
            }

            if leader_key_ref.borrow().is_composing() {
                if key == Key::q {
                    println!("[browser] Quitting...");

                    window_ref.borrow().application().unwrap().quit();
                    return Propagation::Stop;
                }

                if key == Key::n {
                    println!("[browser] New tab...");

                    let url = HOME_DEFAULT;
                    let webview = WebView::new();

                    webview.load_uri(url);
                    webviews_ref.borrow_mut().push(webview);

                    let tab_view = tab_bar.view().unwrap();

                    let index = webviews_ref.borrow().len() - 1;
                    tab_view.append(&webviews_ref.borrow()[index]);

                    let tab_page = tab_view.page(&webviews_ref.borrow()[index]);
                    tab_view.set_selected_page(&tab_page);

                    webviews_ref.borrow()[index].grab_focus();

                    println!("Webviews: {:?}", webviews_ref.borrow().len());

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
