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
    web_view: WebView,
    inspector_visible: bool,
}

impl Page {
    fn new(developer: bool) -> Rc<RefCell<Self>> {
        let web_view = WebView::new();
        let inspector_visible = false;

        if developer {
            let settings = WebViewExt::settings(&web_view).unwrap();
            settings.set_enable_developer_extras(developer);
        }

        let page = Rc::new(RefCell::new(Self {
            web_view,
            inspector_visible,
        }));

        page.borrow()
            .web_view
            .connect_load_changed(move |web_view, event| {
                Self::loaded(web_view, event);
            });

        // FIXME: I give up on this, it's not working
        // this means when the inspector is closed using the "x"
        // the inspector_visible is not updated, the following
        // code attempts to fix that, but it fails.
        //
        // let page_clone = Rc::clone(&page);
        // page.borrow()
        //     .web_view
        //     .inspector()
        //     .unwrap()
        //     .connect_closed(move |_| {
        //         page_clone.borrow_mut().update_inspector_state(false);
        //     });
        //
        // let page_clone = Rc::clone(&page);
        // page.borrow()
        //     .web_view
        //     .inspector()
        //     .unwrap()
        //     .connect_inspected_uri_notify(move |_| {
        //         page_clone.borrow_mut().update_inspector_state(true);
        //     });

        let web_view_key_pressed_controller = EventControllerKey::new();
        let page_clone = Rc::clone(&page);
        web_view_key_pressed_controller.connect_key_pressed(
            move |event, key, keycode, modifier_state| {
                let page_clone = Rc::clone(&page_clone);
                Page::webkit_kb_input(page_clone, event, key, keycode, modifier_state)
            },
        );
        page.borrow()
            .web_view
            .add_controller(web_view_key_pressed_controller);

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

    fn loaded(web_view: &WebView, event: LoadEvent) {
        if event != LoadEvent::Finished {
            return;
        }

        Self::run_js(
            web_view,
            include_str!("vimium/lib/handler_stack.js"),
            |_| {},
        );
        Self::run_js(web_view, include_str!("vimium/lib/dom_utils.js"), |_| {});
        Self::run_js(web_view, include_str!("vimium/lib/utils.js"), |_| {});
        Self::run_js(
            web_view,
            include_str!("vimium/content_scripts/scroller.js"),
            |_| {},
        );
        Self::run_js(web_view, "Scroller.init()", |_| {});
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
        Self::run_js(&self.web_view, javascript.as_str(), |_| {});
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
        Self::run_js(&self.web_view, javascript.as_str(), |_| {});
    }

    fn scroll_up(&self, times: u8) {
        let javascript = format!("Scroller.scrollBy('y', -1 * {} * {})", SCROLL_AMOUNT, times);
        Self::run_js(&self.web_view, javascript.as_str(), |_| {});
    }

    fn scroll_right(&self, times: u8) {
        let javascript = format!("Scroller.scrollBy('x', -1 * {} * {}", SCROLL_AMOUNT, times);
        Self::run_js(&self.web_view, javascript.as_str(), |_| {});
    }

    fn scroll_left(&self, times: u8) {
        let javascript = format!("Scroller.scrollBy('x', {} * {}", SCROLL_AMOUNT, times);
        Self::run_js(&self.web_view, javascript.as_str(), |_| {});
    }

    fn insert_mode<F: Fn(Result<javascriptcore::Value, glib::Error>) + 'static>(&self, f: F) {
        let javascript = "document.activeElement.tagName === 'INPUT' || document.activeElement.tagName === 'TEXTAREA'";
        Self::run_js(&self.web_view, javascript, f);
    }

    fn webkit_kb_input(
        page: Rc<RefCell<Self>>,
        event: &EventControllerKey,
        key: Key,
        keycode: u32,
        modifier_state: ModifierType,
    ) -> Propagation {
        _ = (event, keycode);
        print!("[webkit] ");

        page.borrow().show_key_press(key, modifier_state, true);

        let page_clone: Rc<RefCell<Self>> = Rc::clone(&page);
        page.borrow().insert_mode(move |res| {
            let page_clone = Rc::clone(&page_clone);
            if let Ok(value) = res {
                if value.to_boolean() {
                    // Scrool keys with ctrl + h, j, k, l
                    if key == Key::h && modifier_state.contains(ModifierType::CONTROL_MASK) {
                        page_clone.borrow().scroll_left(1);
                    }
                    if key == Key::j && modifier_state.contains(ModifierType::CONTROL_MASK) {
                        page_clone.borrow().scroll_down(1);
                    }
                    if key == Key::k && modifier_state.contains(ModifierType::CONTROL_MASK) {
                        page_clone.borrow().scroll_up(1);
                    }
                    if key == Key::l && modifier_state.contains(ModifierType::CONTROL_MASK) {
                        page_clone.borrow().scroll_right(1);
                    }
                    // Back / Forward with ctrl + h, l
                    if key == Key::H && modifier_state.contains(ModifierType::CONTROL_MASK) {
                        page_clone.borrow().web_view.go_back();
                    }
                    if key == Key::L && modifier_state.contains(ModifierType::CONTROL_MASK) {
                        page_clone.borrow().web_view.go_forward();
                    }
                } else {
                    // Scrool keys with h, j, k, l
                    if key == Key::h {
                        page_clone.borrow().scroll_left(1);
                    }
                    if key == Key::j {
                        page_clone.borrow().scroll_down(1);
                    }
                    if key == Key::k {
                        page_clone.borrow().scroll_up(1);
                    }
                    if key == Key::l {
                        page_clone.borrow().scroll_right(1);
                    }
                    // Back / Forward with h, l
                    if key == Key::H {
                        page_clone.borrow().web_view.go_back();
                    }
                    if key == Key::L {
                        page_clone.borrow().web_view.go_forward();
                    }
                }
            }
        });

        // Reload
        if key == Key::r && modifier_state.contains(ModifierType::CONTROL_MASK) {
            page.borrow().web_view.reload();

            return Propagation::Stop;
        }

        // Reload harder.
        if key == Key::R && modifier_state.contains(ModifierType::CONTROL_MASK) {
            page.borrow().web_view.reload_bypass_cache();

            return Propagation::Stop;
        }

        // Toggle inspector
        if key == Key::I && modifier_state.contains(ModifierType::CONTROL_MASK) {
            // Page::toggle_inspector(Rc::clone(&page));
            page.borrow_mut().toggle_inspector();

            // Prevents GTK inspector from showing up
            return Propagation::Stop;
        }

        // Close window with Ctrl+w
        if key == Key::w && modifier_state.contains(ModifierType::CONTROL_MASK) {
            page.borrow().web_view.try_close();

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

        // Toggle inspector
        let page_clone = Rc::clone(&page);
        if key == Key::I && modifier_state.contains(ModifierType::CONTROL_MASK) {
            // Page::toggle_inspector(Rc::clone(&page));
            page_clone.borrow_mut().toggle_inspector();

            // Prevents GTK inspector from showing up
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

        let page_clone = Rc::clone(&page);
        if key == Key::from_name("Escape").unwrap() && page_clone.borrow().inspector_visible {
            // Page::toggle_inspector(page_clone);
            page_clone.borrow_mut().toggle_inspector();

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
    pages: Vec<Rc<RefCell<Page>>>,
}

impl Browser {
    fn new(app: &Application) -> Rc<RefCell<Self>> {
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

        let browser = Rc::new(RefCell::new(Self {
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

    // fn close(&self) {
    //     todo!("Close tab");
    // }

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
        let url = HOME_DEFAULT;
        let developer = true;

        let page = Page::new(developer);
        page.borrow().load_url(url);

        self.pages.push(page);

        let tab_view = self.tab_bar.view().unwrap();

        let index = self.pages.len() - 1;
        tab_view.append(&self.pages[index].borrow().web_view);

        let tab_page = tab_view.page(&self.pages[index].borrow().web_view);
        tab_view.set_selected_page(&tab_page);

        let page_clone = Rc::clone(&self.pages[index]);
        let tab_page_clone = tab_page.clone();
        self.pages[index]
            .borrow()
            .web_view
            .connect_load_changed(move |_, _| {
                let tab_page_clone = tab_page_clone.clone();

                Page::run_js(
                    &page_clone.borrow().web_view,
                    "document.title",
                    move |res| {
                        if let Ok(value) = res {
                            let title = value.to_string();
                            tab_page_clone.set_title(title.as_str());
                        }
                    },
                );
            });

        self.pages[index].borrow().web_view.grab_focus();
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
        if key == self.leader_key.borrow().key {
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
    Browser::new(app).borrow_mut().show();
}

fn main() -> glib::ExitCode {
    let application = Application::builder()
        .application_id("com.igorgue.Browser")
        .build();

    application.connect_activate(activate);

    application.run()
}
