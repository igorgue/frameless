const std = @import("std");
const c = @cImport({
    @cInclude("libadwaita-1/adwaita.h");
    @cInclude("gtk-4.0/gdk/gdk.h");
    @cInclude("webkitgtk-6.0/webkit/webkit.h");
});

const home = "https://ziglang.org/";

fn input(widget: *c.GtkWidget, keyval: c.guint, keycode: c.guint, state: *c.GdkModifierType, event_controller: *c.GtkEventControllerKey) callconv(.C) c.gboolean {
    _ = widget;
    _ = state;
    _ = event_controller;

    std.debug.print("key val {any}\n", .{keyval});
    std.debug.print("key code {any}\n", .{keycode});

    return 0;
}

fn activate(app: *c.GtkApplication, user_data: c.gpointer) callconv(.C) void {
    _ = user_data;

    const window = c.gtk_application_window_new(app);
    const web_view = c.webkit_web_view_new();
    const key_press_event_controller = c.gtk_event_controller_key_new();

    _ = c.g_signal_connect_object(
        key_press_event_controller,
        "key-pressed",
        @as(c.GCallback, @ptrCast(&input)),
        window,
        c.G_CONNECT_SWAPPED,
    );

    _ = c.g_signal_connect_object(
        key_press_event_controller,
        "key-pressed",
        @as(c.GCallback, @ptrCast(&input)),
        web_view,
        c.G_CONNECT_SWAPPED,
    );

    // const overlay = c.gtk_overlay_new();
    // const button = c.gtk_button_new();

    c.gtk_widget_add_controller(@as(*c.GtkWidget, @ptrCast(window)), key_press_event_controller);

    c.gtk_window_set_default_size(@as(*c.GtkWindow, @ptrCast(window)), 200, 200);
    // c.gtk_overlay_set_child(@as(*c.GtkOverlay, @ptrCast(overlay)), web_view);
    c.gtk_window_set_child(@as(*c.GtkWindow, @ptrCast(window)), web_view);
    // c.gtk_overlay_add_overlay(@as(*c.GtkOverlay, @ptrCast(overlay)), button);
    c.gtk_window_present(@as(*c.GtkWindow, @ptrCast(window)));

    // on click for trhe label reveal the web view
    // _ = c.g_signal_connect_data(
    //     label,
    //     "button-press-event",
    //     @as(c.GCallback, @ptrCast(&|_, _| {
    //         c.gtk_overlay_set_overlay_pass_through(@as(*c.GtkOverlay, @ptrCast(overlay)), web_view, true);
    //         c.gtk_widget_hide(@as(*c.GtkWidget, @ptrCast(label)));
    //         return false;
    //     })),
    //     null,
    //     null,
    //     0,
    // );

    c.webkit_web_view_load_uri(@as(*c.WebKitWebView, @ptrCast(web_view)), home);
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
