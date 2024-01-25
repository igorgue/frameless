use adw::prelude::*;

use adw::{Application, ApplicationWindow};

use webkit::{prelude::*,WebView};

fn main() {
    let application = Application::builder()
        .application_id("com.igorgue.Browser")
        .build();

    application.connect_activate(|app| {
        let webview = WebView::new();
        webview.load_uri("https://crates.io/");

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Browser")
            .default_width(350)
            // add content to window
            .content(&webview)
            .build();
        window.present();
    });

    application.run();
}
