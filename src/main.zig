const std = @import("std");
const c = @cImport({
    @cInclude("libadwaita-1/adwaita.h");
    @cInclude("webkitgtk-6.0/webkit/webkit.h");
});

const home = "https://ziglang.org/";

fn activateCb(app: *c.GtkApplication, data: c.gpointer) callconv(.C) void {
    _ = data;

    const window = c.gtk_application_window_new(app);
    const web_view = c.webkit_web_view_new();
    const overlay = c.gtk_overlay_new();
    const button = c.gtk_button_new();

    c.gtk_window_set_default_size(@as(*c.GtkWindow, @ptrCast(window)), 200, 200);
    c.gtk_overlay_set_child(@as(*c.GtkOverlay, @ptrCast(overlay)), web_view);
    c.gtk_window_set_child(@as(*c.GtkWindow, @ptrCast(window)), overlay);
    c.gtk_overlay_add_overlay(@as(*c.GtkOverlay, @ptrCast(overlay)), button);
    c.gtk_window_present(@as(*c.GtkWindow, @ptrCast(window)));

    // on click for trhe label reveal the web view
    _ = c.g_signal_connect_data(
        label,
        "button-press-event",
        @as(c.GCallback, @ptrCast(&|_, _| {
            c.gtk_overlay_set_overlay_pass_through(@as(*c.GtkOverlay, @ptrCast(overlay)), web_view, true);
            c.gtk_widget_hide(@as(*c.GtkWidget, @ptrCast(label)));
            return false;
        })),
        null,
        null,
        0,
    );

    c.webkit_web_view_load_uri(@as(*c.WebKitWebView, @ptrCast(web_view)), home);
}

pub fn main() void {
    const app = c.adw_application_new("org.igorgue.Browser", c.G_APPLICATION_DEFAULT_FLAGS);

    _ = c.g_signal_connect_data(
        app,
        "activate",
        @as(c.GCallback, @ptrCast(&activateCb)),
        null,
        null,
        0,
    );

    _ = c.g_application_run(@ptrCast(app), 0, null);
}
