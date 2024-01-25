const std = @import("std");
const c = @cImport({
    @cInclude("libadwaita-1/adwaita.h");
    @cInclude("gtk-4.0/gdk/gdk.h");
    @cInclude("webkitgtk-6.0/webkit/webkit.h");
});

const HOME = "https://ziglang.org/";

fn input(widget: *c.GtkWidget, keyval: c.guint, keycode: c.guint, state: *c.GdkModifierType, event_controller: *c.GtkEventControllerKey) callconv(.C) c.gboolean {
    _ = widget;
    _ = state;
    _ = event_controller;

    std.debug.print("key val {any}\n", .{keyval});
    std.debug.print("key code {any}\n", .{keycode});

    return 0;
}

fn loadChanged(web_view: *c.WebKitWebView, load_event: c.WebKitLoadEvent, user_data: c.gpointer) callconv(.C) void {
    _ = user_data;

    if (load_event == c.WEBKIT_LOAD_FINISHED) {
        const javascript = "alert(document.title);"; // Your JavaScript code

        c.webkit_web_view_evaluate_javascript(web_view, javascript, javascript.len, null, null, null, null, null);
    }
}

fn activate(app: *c.GtkApplication, user_data: c.gpointer) callconv(.C) void {
    _ = user_data;

    const window = c.gtk_application_window_new(app);
    const key_press_event_controller = c.gtk_event_controller_key_new();

    const web_view = c.webkit_web_view_new();
    const inspector = c.webkit_web_view_get_inspector(@as(*c.WebKitWebView, @ptrCast(web_view)));
    // const user_content_manager = c.webkit_web_view_get_user_content_manager(@as(*c.WebKitWebView, @ptrCast(web_view)));
    // const javascript = "window.alert('Hello from JavaScript!');";
    // const user_script = c.webkit_user_script_new(javascript, c.WEBKIT_USER_CONTENT_INJECT_TOP_FRAME, c.WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_START, null, null);

    // c.webkit_user_content_manager_add_script(user_content_manager, user_script);
    // c.webkit_user_script_unref(user_script);
    //
    // Connect the 'load-changed' signal
    _ = c.g_signal_connect_data(
        web_view,
        "load-changed",
        @as(c.GCallback, @ptrCast(&loadChanged)),
        null,
        null,
        0,
    );

    _ = c.g_signal_connect_object(
        key_press_event_controller,
        "key-pressed",
        @as(c.GCallback, @ptrCast(&input)),
        window,
        c.G_CONNECT_SWAPPED,
    );

    c.gtk_widget_add_controller(@as(*c.GtkWidget, @ptrCast(window)), key_press_event_controller);

    c.gtk_window_set_default_size(@as(*c.GtkWindow, @ptrCast(window)), 200, 200);
    c.gtk_window_set_child(@as(*c.GtkWindow, @ptrCast(window)), web_view);
    c.gtk_window_present(@as(*c.GtkWindow, @ptrCast(window)));

    c.webkit_web_view_load_uri(@as(*c.WebKitWebView, @ptrCast(web_view)), HOME);
    c.webkit_web_inspector_show(inspector);
}

pub fn main() void {
    const app = c.adw_application_new("org.igorgue.Browser", c.G_APPLICATION_DEFAULT_FLAGS);

    _ = c.g_signal_connect_data(
        app,
        "activate",
        @as(c.GCallback, @ptrCast(&activate)),
        null,
        null,
        0,
    );

    _ = c.g_application_run(@ptrCast(app), 0, null);
}
