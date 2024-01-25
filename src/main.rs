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
            .title("First App")
            .default_width(350)
            // add content to window
            .content(&webview)
            .build();
        window.present();
    });

    application.run();
}
// use adw::gtk::{glib, prelude::*, ApplicationWindow};
// use webkit::{prelude::*, WebView};
//
// fn main() -> glib::ExitCode {
//     let app = adw::gtk::Application::new(Some("org.gnome.webkit6-rs.example"), Default::default());
//     app.connect_activate(move |app| {
//         let window = ApplicationWindow::new(app);
//         let webview = WebView::new();
//         webview.load_uri("https://crates.io/");
//         window.set_child(Some(&webview));
//
//         let settings = WebViewExt::settings(&webview).unwrap();
//         settings.set_enable_developer_extras(true);
//
//         let inspector = webview.inspector().unwrap();
//         inspector.show();
//
//         webview.evaluate_javascript(
//             "alert('Hello');",
//             None,
//             None,
//             adw::gtk::gio::Cancellable::NONE,
//             |_result| {},
//         );
//         webview.evaluate_javascript("42", None, None, adw::gtk::gio::Cancellable::NONE, |result| {
//             match result {
//                 Ok(value) => {
//                     println!("is_boolean: {}", value.is_boolean());
//                     println!("is_number: {}", value.is_number());
//                     println!("{:?}", value.to_boolean());
//                 }
//                 Err(error) => println!("{}", error),
//             }
//         });
//         window.present();
//     });
//     app.run()
// }
