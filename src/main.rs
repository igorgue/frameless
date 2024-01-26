use adw::gdk::{Key, ModifierType};
use adw::gio::Cancellable;
use adw::glib::Propagation;
use adw::gtk::EventControllerKey;
use adw::{Application, ApplicationWindow};
use webkit::{prelude::*, LoadEvent, WebView};

fn input(event: &EventControllerKey, key: Key, keycode: u32, state: ModifierType) -> Propagation {
    _ = (event, key, state);

    eprintln!("Key pressed: {:?}", keycode);

    Propagation::Stop
}

fn loaded(webview: &WebView, event: LoadEvent) {
    if event != LoadEvent::Finished {
        return;
    }

    eprintln!("Loaded: {:?}", event);

    let javascript = "document.body.style.backgroundColor = 'red';";
    let cancellable: Option<&Cancellable> = None;
    webview.evaluate_javascript(javascript, None, None, cancellable, |_| {});
}

fn activate(app: &Application) {
    let key_pressed_controller = EventControllerKey::new();
    key_pressed_controller.connect_key_pressed(input);

    let webview = WebView::new();
    webview.load_uri("https://crates.io/");

    webview.connect_load_changed(loaded);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Browser")
        .default_width(350)
        .content(&webview)
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
