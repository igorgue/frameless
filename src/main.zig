const c = @cImport({
    @cInclude("libadwaita-1/adwaita.h");
    @cInclude("webkitgtk-6.0/webkit/webkit.h");
    @cInclude("gio/gio.h");
});
const c_true = 1;
const c_false = 0;

pub fn main() !void {
    c.adw_init();

    const window = c.adw_window_new();
    const web_view = c.webkit_web_view_new();
    // const quit_action = c.GSimpleAction;
    // const quit_action = c.g_simple_action_new("quit", null);

    // const callback = c.gtk_callback_action_new("quit", c.G_CALLBACK(@ptrCast(fn() void {
    //     c.g_application_quit(@as(c.GApplication, @ptrCast(c.g_application_get_default())))
    // })), null, null);
    //
    // _ = callback;

    // c.g_signal_connect_data(@ptrCast(window), "destroy", c.GCallback(@ptrCast(fn() void {
    //     c.g_application_quit(@as(c.GApplication, @ptrCast(c.g_application_get_default())))
    // })), null, null, 0);
    // _ = quit_action;

    c.gtk_window_set_default_size(@as(*c.GtkWindow, @ptrCast(window)), 200, 200);
    c.adw_window_set_content(@as(*c.AdwWindow, @ptrCast(window)), web_view);
    c.gtk_window_present(@as(*c.GtkWindow, @ptrCast(window)));

    c.webkit_web_view_load_uri(@as(*c.WebKitWebView, @ptrCast(web_view)), "https://ziglang.org/");

    while (c.g_list_model_get_n_items(c.gtk_window_get_toplevels()) > 0)
        _ = c.g_main_context_iteration(null, c_true);
}
