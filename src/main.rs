use std::cell::RefCell;
use std::include_str;
use std::rc::Rc;
use std::time::SystemTime;

use adw::gdk::{Key, ModifierType};
use adw::gio::Cancellable;
use adw::glib::Propagation;
use adw::gtk::EventControllerKey;
use adw::prelude::*;
use adw::{Application, ApplicationWindow};
use webkit::{glib, javascriptcore, prelude::*, LoadEvent, WebView};

const LEADER_KEY_DEFAULT: Key = Key::semicolon;
const LEADER_KEY_COMPOSE_TIME: u64 = 500; // ms
const SCROLL_AMOUNT: i32 = 40;
const HOME_DEFAULT: &str = "https://crates.io";

#[derive(Debug)]
struct Page {
    title: String,
    web_view: WebView,
    inspector_visible: bool,
}

impl Page {
    fn new(index: usize, developer: bool) -> Self {
        let web_view = WebView::new();
        let inspector_visible = false;
        let title = String::new();

        if developer {
            let settings = WebViewExt::settings(&web_view).unwrap();
            settings.set_enable_developer_extras(developer);
        }

        Self {
            web_view,
            title,
            inspector_visible,
        }
    }

    fn load_url<'a>(&'a self, url: &str) {
        self.web_view.load_uri(url);

        self.loaded(&self.web_view, LoadEvent::Finished);

        // self.web_view.connect_load_changed(move |webview, event| {
        //     self.loaded(webview, event);
        // });
        //
        // self.web_view.inspector().unwrap().connect_closed(move |_| {
        //     self.borrow_mut().inspector_visible = false;
        // });
    }

    fn toggle_inspector(&mut self) {
        if self.inspector_visible {
            self.web_view.inspector().unwrap().close();
            self.inspector_visible = false;
        } else {
            self.web_view.inspector().unwrap().show();
            self.inspector_visible = true;
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

    fn console_log(&self, message: &str) {
        let javascript = format!("console.log('{}')", message);
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

    fn insert_mode<F: Fn(Result<javascriptcore::Value, glib::Error>) + 'static>(&self, f: F) {
        let javascript = "document.activeElement.tagName === 'INPUT' || document.activeElement.tagName === 'TEXTAREA'";
        self.run_js(javascript, f);
    }

    fn webkit_kb_input(
        &mut self,
        browser: &mut Browser,
        event: &EventControllerKey,
        key: Key,
        keycode: u32,
        modifier_state: ModifierType,
    ) -> Propagation {
        _ = (event, keycode);
        print!("[web_view] ");

        self.show_key_press(key, modifier_state, true);

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
            self.toggle_inspector();

            // Prevents GTK inspector from showing up
            return Propagation::Stop;
        }

        // Close window with Ctrl+w
        if key == Key::w && modifier_state.contains(ModifierType::CONTROL_MASK) {
            browser.close();

            return Propagation::Stop;
        }

        // Handle leader key
        // if key == self.browser.leader_key.borrow().key {
        //     let leader_key_clone = Rc::clone(&self.browser.leader_key);
        //
        //     // FIXME: Propagation::Stop is not returned here...
        //     // but should.
        //     self.insert_mode(move |res| {
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
        // } else if self.browser.leader_key.borrow().is_composing() {
        //     if key == Key::q {
        //         println!("[browser] Quitting...");
        //         self.browser.window.application().unwrap().quit();
        //
        //         return Propagation::Stop;
        //     }
        //
        //     return Propagation::Stop;
        // }

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
            self.toggle_inspector();

            return Propagation::Stop;
        }

        Propagation::Proceed
    }
}

#[derive(Debug)]
struct Browser {
    leader_key: Rc<RefCell<LeaderKey>>,
    window: ApplicationWindow,
    tab_bar: adw::TabBar,
    pages: Vec<Page>,
}

impl Browser {
    fn new(app: &Application) -> Rc<RefCell<Self>> {
        let tab_bar = adw::TabBar::builder().build();
        // let web_view = WebView::new();
        let tab_view = adw::TabView::builder().build();

        // tab_view.append(&web_view);
        // let page = tab_view.page(&web_view);
        // tab_view.set_selected_page(&page);

        // let another_web_view = WebView::new();
        // tab_view.append(&another_web_view);
        // let another_page = tab_view.page(&another_web_view);
        // another_web_view.load_uri("https://www.rust-lang.org");
        // tab_view.set_selected_page(&another_page);

        tab_bar.set_view(Some(&tab_view));

        // let header_bar = adw::HeaderBar::new();
        //
        // let title_url_entry = adw::gtk::Entry::new();
        // title_url_entry.set_placeholder_text(Some("Search or enter address"));
        // title_url_entry.set_hexpand(true);
        // title_url_entry.set_vexpand(true);
        //
        // header_bar.set_title_widget(Some(&title_url_entry));
        let toolbar_view = adw::ToolbarView::new();

        // toolbar_view.add_top_bar(&header_bar);
        toolbar_view.add_top_bar(&tab_bar);
        toolbar_view.set_content(Some(&tab_view));

        // let container = adw::gtk::Box::new(adw::gtk::Orientation::Vertical, 0);
        // container.append(&web_view);

        // toolbar_view.add_bottom_bar(&container);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Browser")
            .default_width(350)
            .content(&toolbar_view)
            .build();

        let browser = Rc::new(RefCell::new(Self {
            // home: std::env::var("BROWSER_HOME").unwrap_or(HOME_DEFAULT.to_string()),
            leader_key: Rc::new(RefCell::new(LeaderKey::new(LEADER_KEY_DEFAULT, 0))),
            window,
            tab_bar,
            pages: vec![],
        }));

        let browser_clone = Rc::clone(&browser);
        let window_key_pressed_controller = EventControllerKey::new();
        window_key_pressed_controller.connect_key_pressed(
            move |event, key, keycode, modifier_state| {
                let mut browser = browser_clone.borrow_mut();

                browser.window_kb_input(event, key, keycode, modifier_state);

                Propagation::Proceed
            },
        );
        browser
            .borrow()
            .window
            .add_controller(window_key_pressed_controller);

        browser
    }

    fn show(&self) {
        self.window.present();
    }

    fn quit(&self) {
        self.window.application().unwrap().quit();
    }

    fn close(&self) {
        todo!("Close tab");
    }

    fn update_leader_key(&mut self, key: Key) {
        self.leader_key = Rc::new(RefCell::new(LeaderKey::new(key, get_current_time())));
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

    fn new_tab(&mut self) {
        let page = Page::new(self.pages.len(), true);

        page.load_url(HOME_DEFAULT);
        self.pages.push(page);

        let tab_view = self.tab_bar.view().unwrap();
        tab_view.append(&self.pages[self.pages.len() - 1].web_view);

        let tab_page = tab_view.page(&self.pages[self.pages.len() - 1].web_view);
        tab_view.set_selected_page(&tab_page);
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

            if key == Key::n {
                println!("[browser] New tab...");

                self.new_tab();

                return Propagation::Stop;
            }

            return Propagation::Stop;
        }

        Propagation::Proceed
    }
}

#[derive(Debug)]
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
    let browser = Browser::new(app);
    let browser_ref = browser.borrow_mut();

    browser_ref.show();

    // let page = Page::new(0, true);
    //
    // page.load_url(HOME_DEFAULT);
    //
    // browser.pages.push(page);
    //
    // let tab_view = browser.tab_bar.view().unwrap();
    //
    // tab_view.append(&browser.pages[0].web_view);
    // let tab_page = tab_view.page(&browser.pages[0].web_view);
    // tab_view.set_selected_page(&tab_page);
    //
    // browser.tab_bar.show();

    // let web_view_key_pressed_controller = EventControllerKey::new();
    // let browser_clone = Rc::clone(&browser);
    // web_view_key_pressed_controller.connect_key_pressed(
    //     move |event, key, keycode, modifier_state| {
    //         browser_clone
    //             .borrow_mut()
    //             .webkit_kb_input(event, key, keycode, modifier_state)
    //     },
    // );
    // browser
    //     .borrow()
    //     .web_view
    //     .add_controller(web_view_key_pressed_controller);
    // browser
    //     .borrow()
    //     .web_view
    //     .load_uri(browser.borrow().home.as_str());
    // let browser_clone = Rc::clone(&browser);
    // browser
    //     .borrow()
    //     .web_view
    //     .connect_load_changed(move |webview, event| {
    //         browser_clone.borrow_mut().loaded(webview, event);
    //     });
    //
    // let settings = WebViewExt::settings(&browser.borrow().web_view).unwrap();
    // settings.set_enable_developer_extras(true);

    // browser.borrow().window.present();

    // let browser_clone = Rc::clone(&browser);
    // browser.borrow().inspector.connect_closed(move |_| {
    //     browser_clone.borrow_mut().inspector_visible = false;
    // });
}

fn main() {
    let application = Application::builder()
        .application_id("com.igorgue.Browser")
        .build();

    application.connect_activate(activate);

    application.run();
}
