// Native, process-owned props only. No foreign-window or global-input API.
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif
#include "host.h"
#include <gtk/gtk.h>
#include <gdk/x11/gdkx.h>
#include <gdk/wayland/gdkwayland.h>
#include <glib-unix.h>
#include <X11/Xutil.h>
#include <fontconfig/fontconfig.h>
#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <sys/mman.h>

#define PROP_LIMIT 8
#define INPUT_LIMIT (4 * 1024 * 1024)
#define OUTPUT_LIMIT 512

typedef struct {
    GtkWindow *window;
    GtkWidget *editor;
    cairo_surface_t *image;
    unsigned char *pixels;
    uint64_t id;
    unsigned kind;
    int x, y, width, height, max_width, max_height;
    unsigned pixel_width, pixel_height;
    gboolean reported;
} Prop;

static Prop props[PROP_LIMIT];
static GMainLoop *loop;
static GByteArray *input;
static gboolean positioning, failed, stopping;
static uint64_t last_id;
static cairo_user_data_key_t image_pixels_key;

_Static_assert(sizeof(HonkPropCommand) == 72, "Zig/C command ABI must agree");

static void fail(void) {
    failed = TRUE;
    if (loop) g_main_loop_quit(loop);
}

// Every event fits PIPE_BUF. Backpressure is a failed private connection; never
// let a blocked output pipe freeze the GTK thread or grow an unbounded queue.
static void emit(const char *message) {
    size_t size = strlen(message);
    if (size >= OUTPUT_LIMIT) { fail(); return; }
    ssize_t written;
    do { written = write(STDOUT_FILENO, message, size); } while (written < 0 && errno == EINTR);
    if (written != (ssize_t)size) fail();
}

static GdkSurface *surface_for(Prop *prop) {
    if (!prop->window) return NULL;
    return gtk_native_get_surface(GTK_NATIVE(prop->window));
}

static Window owned_xid(Prop *prop, Display **display) {
    GdkSurface *surface = surface_for(prop);
    if (!surface || gdk_surface_is_destroyed(surface) || !GDK_IS_X11_SURFACE(surface)) return None;
    *display = gdk_x11_display_get_xdisplay(gdk_surface_get_display(surface));
    return gdk_x11_surface_get_xid(surface);
}

static void report(Prop *prop, const char *origin) {
    char message[OUTPUT_LIMIT];
    g_snprintf(message, sizeof(message),
        "{\"v\":1,\"event\":\"window\",\"id\":%" G_GUINT64_FORMAT
        ",\"kind\":%u,\"x\":%d,\"y\":%d,\"width\":%d,\"height\":%d,\"alive\":%s,\"origin\":%s}\n",
        (guint64)prop->id, prop->kind, prop->x, prop->y, prop->width, prop->height,
        origin ? "false" : "true", origin ? origin : "null");
    emit(message);
    prop->reported = TRUE;
}

static void close_prop(Prop *prop, gboolean user) {
    GtkWindow *window = prop->window;
    if (!window) return;
    if (!stopping) report(prop, user ? "\"user\"" : "\"program\"");
    prop->window = NULL;
    gtk_window_destroy(window);
    if (prop->image) cairo_surface_destroy(prop->image);
    g_free(prop->pixels);
    memset(prop, 0, sizeof(*prop));
}

static gboolean user_close(GtkWindow *window, gpointer data) {
    (void)window;
    close_prop(data, TRUE);
    return TRUE;
}

static void close_clicked(GtkButton *button, gpointer data) {
    (void)button;
    close_prop(data, TRUE);
}

static void paint_image(GtkDrawingArea *area, cairo_t *cr, int width, int height, gpointer data) {
    Prop *prop = data;
    if (!prop->image) return;
    // GTK allocates logical units. Limit physical output to the source pixels
    // even when the window's scale factor changes, and retain the entire image.
    double scale = MIN(1.0 / gtk_widget_get_scale_factor(GTK_WIDGET(area)),
                       MIN((double)width / prop->pixel_width, (double)height / prop->pixel_height));
    cairo_translate(cr, (width - prop->pixel_width * scale) / 2.0,
                        (height - prop->pixel_height * scale) / 2.0);
    cairo_scale(cr, scale, scale);
    cairo_set_source_surface(cr, prop->image, 0, 0);
    cairo_pattern_set_filter(cairo_get_source(cr), CAIRO_FILTER_BEST);
    cairo_paint(cr);
}

static void update_geometry(Prop *prop) {
    GdkSurface *surface = surface_for(prop);
    if (!surface || gdk_surface_is_destroyed(surface) || !gdk_surface_get_mapped(surface)) return;
    int x = 0, y = 0;
    // Native Wayland engine coordinates are logical; X11 uses physical pixels.
    int width = gdk_surface_get_width(surface);
    int height = gdk_surface_get_height(surface);
    if (positioning) {
        Display *display = NULL;
        Window window = owned_xid(prop, &display), child = None;
        XWindowAttributes attributes;
        if (window == None || !XGetWindowAttributes(display, window, &attributes) ||
            !XTranslateCoordinates(display, window, DefaultRootWindow(display), 0, 0, &x, &y, &child)) {
            fail(); return;
        }
        width = attributes.width;
        height = attributes.height;
    }
    // GTK's minimum natural size must never quietly defeat the monitor ceiling.
    if (width < 1 || height < 1 || width > prop->max_width || height > prop->max_height) {
        fail(); return;
    }
    if (!prop->reported || x != prop->x || y != prop->y || width != prop->width || height != prop->height) {
        prop->x = x; prop->y = y; prop->width = width; prop->height = height;
        report(prop, NULL);
    }
}

static void move_prop(Prop *prop, int x, int y) {
    Display *display = NULL;
    Window window = owned_xid(prop, &display);
    if (window == None) { fail(); return; }
    XMoveWindow(display, window, x, y);
    XFlush(display);
}

static int spawn_prop(Prop *prop, const HonkPropCommand *command) {
    prop->id = command->id;
    prop->kind = command->operation;
    prop->max_width = (int)command->width;
    prop->max_height = (int)command->height;
    prop->width = prop->max_width;
    prop->height = prop->max_height;
    prop->window = GTK_WINDOW(gtk_window_new());
    gtk_window_set_title(prop->window, prop->kind == HONK_PROP_NOTE ? "Honk300 note" : "Honk300 picture");
    gtk_window_set_decorated(prop->window, FALSE);
    gtk_window_set_resizable(prop->window, FALSE);
    g_signal_connect(prop->window, "close-request", G_CALLBACK(user_close), prop);
    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    GtkWidget *header = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 4);
    gtk_widget_add_css_class(header, "honk-prop-header");
    GtkWidget *label = gtk_label_new(command->title);
    gtk_label_set_ellipsize(GTK_LABEL(label), PANGO_ELLIPSIZE_END);
    gtk_label_set_max_width_chars(GTK_LABEL(label), 1);
    gtk_widget_set_hexpand(label, TRUE);
    gtk_widget_set_margin_start(label, 8);
    gtk_widget_set_halign(label, GTK_ALIGN_FILL);
    GtkWidget *close = gtk_button_new_with_label("Close");
    gtk_widget_add_css_class(close, "honk-prop-close");
    gtk_accessible_update_property(GTK_ACCESSIBLE(close), GTK_ACCESSIBLE_PROPERTY_LABEL, "Close", -1);
    g_signal_connect(close, "clicked", G_CALLBACK(close_clicked), prop);
    gtk_box_append(GTK_BOX(header), label);
    gtk_box_append(GTK_BOX(header), close);
    GtkWidget *handle = gtk_window_handle_new();
    gtk_window_handle_set_child(GTK_WINDOW_HANDLE(handle), header);
    gtk_box_append(GTK_BOX(box), handle);
    if (prop->kind == HONK_PROP_NOTE) {
        prop->editor = gtk_text_view_new();
        gtk_text_view_set_wrap_mode(GTK_TEXT_VIEW(prop->editor), GTK_WRAP_WORD_CHAR);
        gtk_text_view_set_left_margin(GTK_TEXT_VIEW(prop->editor), 10);
        gtk_text_view_set_right_margin(GTK_TEXT_VIEW(prop->editor), 10);
        gtk_text_view_set_top_margin(GTK_TEXT_VIEW(prop->editor), 8);
        gtk_text_view_set_bottom_margin(GTK_TEXT_VIEW(prop->editor), 8);
        gtk_accessible_update_property(GTK_ACCESSIBLE(prop->editor), GTK_ACCESSIBLE_PROPERTY_LABEL, "Note", -1);
        GtkWidget *scroll = gtk_scrolled_window_new();
        gtk_scrolled_window_set_child(GTK_SCROLLED_WINDOW(scroll), prop->editor);
        gtk_widget_set_vexpand(scroll, TRUE);
        gtk_box_append(GTK_BOX(box), scroll);
    } else {
        prop->pixel_width = command->pixel_width;
        prop->pixel_height = command->pixel_height;
        prop->pixels = g_malloc(command->pixels_len);
        // tiny-skia sends premultiplied RGBA; Cairo ARGB32 is native-endian.
        for (size_t offset = 0; offset < command->pixels_len; offset += 4) {
            uint32_t pixel = ((uint32_t)command->pixels[offset + 3] << 24) |
                ((uint32_t)command->pixels[offset] << 16) |
                ((uint32_t)command->pixels[offset + 1] << 8) | command->pixels[offset + 2];
            memcpy(prop->pixels + offset, &pixel, sizeof(pixel));
        }
        prop->image = cairo_image_surface_create_for_data(prop->pixels, CAIRO_FORMAT_ARGB32,
            (int)prop->pixel_width, (int)prop->pixel_height, (int)prop->pixel_width * 4);
        if (cairo_surface_status(prop->image) != CAIRO_STATUS_SUCCESS) return 0;
        // A GTK render node may retain the source surface after a window closes.
        // Tie the pixel allocation to Cairo's final reference, not the registry slot.
        if (cairo_surface_set_user_data(prop->image, &image_pixels_key, prop->pixels, g_free) != CAIRO_STATUS_SUCCESS) return 0;
        prop->pixels = NULL;
        GtkWidget *drawing = gtk_drawing_area_new();
        gtk_drawing_area_set_draw_func(GTK_DRAWING_AREA(drawing), paint_image, prop, NULL);
        gtk_widget_set_vexpand(drawing, TRUE);
        gtk_accessible_update_property(GTK_ACCESSIBLE(drawing), GTK_ACCESSIBLE_PROPERTY_LABEL, command->title, -1);
        gtk_box_append(GTK_BOX(box), drawing);
    }
    gtk_window_set_child(prop->window, box);
    gtk_widget_realize(GTK_WIDGET(prop->window));
    int scale = positioning ? gtk_widget_get_scale_factor(GTK_WIDGET(prop->window)) : 1;
    gtk_window_set_default_size(prop->window, MAX(1, (int)command->width / scale),
                               MAX(1, (int)command->height / scale));
    if (positioning) {
        Display *display = NULL;
        Window window = owned_xid(prop, &display);
        if (window == None) return 0;
        XSizeHints hints = {0};
        hints.flags = USPosition;
        hints.x = command->x; hints.y = command->y;
        XSetWMNormalHints(display, window, &hints);
        move_prop(prop, command->x, command->y);
    }
    // Showing an owned delivery must not focus a terminal or synthesize input.
    gtk_widget_set_visible(GTK_WIDGET(prop->window), TRUE);
    if (!positioning) {
        GdkSurface *surface = surface_for(prop);
        if (!surface || !GDK_IS_WAYLAND_TOPLEVEL(surface)) return 0;
        char application_id[80];
        g_snprintf(application_id, sizeof(application_id), "honk300.prop.%" G_GUINT64_FORMAT,
                   (guint64)prop->id);
        // GTK creates the xdg_toplevel while mapping. Its Wayland setter does
        // nothing before that point and does not retain a pending application id.
        gdk_wayland_toplevel_set_application_id(GDK_TOPLEVEL(surface), application_id);
    }
    return !failed;
}

int honk_props_apply(const HonkPropCommand *command) {
    if (command->operation == HONK_PROP_SHUTDOWN) {
        stopping = TRUE;
        g_main_loop_quit(loop);
        return 1;
    }
    Prop *prop = NULL, *free_slot = NULL;
    for (unsigned index = 0; index < PROP_LIMIT; index++) {
        if (props[index].window && props[index].id == command->id) prop = &props[index];
        if (!props[index].window && !free_slot) free_slot = &props[index];
    }
    if (command->operation == HONK_PROP_NOTE || command->operation == HONK_PROP_IMAGE) {
        if (prop || command->id <= last_id) return 0;
        last_id = command->id;
        if (!free_slot) {
            char message[OUTPUT_LIMIT];
            g_snprintf(message, sizeof(message), "{\"v\":1,\"event\":\"busy\",\"id\":%" G_GUINT64_FORMAT "}\n", (guint64)command->id);
            emit(message);
            return !failed;
        }
        return spawn_prop(free_slot, command);
    }
    // A native user close can overtake already-queued commands. A stale opaque
    // token is a no-op; it cannot resolve to another window or be reused.
    if (!prop) return 1;
    switch (command->operation) {
    case HONK_PROP_MOVE:
        if (!positioning) return 0;
        move_prop(prop, command->x, command->y);
        break;
    case HONK_PROP_PASSTHROUGH: {
        GdkSurface *surface = surface_for(prop);
        if (!surface) return 0;
        cairo_region_t *empty = command->passthrough ? cairo_region_create() : NULL;
        gdk_surface_set_input_region(surface, empty);
        if (empty) cairo_region_destroy(empty);
        break;
    }
    case HONK_PROP_FOCUS:
        // Focus only the owned edit widget. Do not request desktop activation;
        // note delivery writes its buffer directly and works without focus.
        if (prop->editor) gtk_widget_grab_focus(prop->editor);
        break;
    case HONK_PROP_TEXT:
        if (!prop->editor) return 0;
        gtk_text_buffer_set_text(gtk_text_view_get_buffer(GTK_TEXT_VIEW(prop->editor)), command->text, -1);
        break;
    case HONK_PROP_CLOSE:
        close_prop(prop, FALSE);
        break;
    default: return 0;
    }
    return !failed;
}

static gboolean consume_commands(void) {
    for (unsigned index = 0; index < 32; index++) {
        unsigned char *newline = memchr(input->data, '\n', input->len);
        if (!newline) break;
        size_t length = newline - input->data;
        if (!length || !honk_props_decode(input->data, length)) { fail(); return FALSE; }
        g_byte_array_remove_range(input, 0, (guint)length + 1);
        if (stopping) return FALSE;
    }
    return TRUE;
}

static gboolean read_commands(gint fd, GIOCondition condition, gpointer unused) {
    (void)unused;
    unsigned char chunk[65536];
    size_t processed = 0;
    for (;;) {
        ssize_t count = read(fd, chunk, sizeof(chunk));
        if (count == 0) { stopping = TRUE; g_main_loop_quit(loop); return G_SOURCE_REMOVE; }
        if (count < 0) {
            if (errno == EINTR) continue;
            if (errno == EAGAIN || errno == EWOULDBLOCK) break;
            fail(); return G_SOURCE_REMOVE;
        }
        if (input->len + count > INPUT_LIMIT) { fail(); return G_SOURCE_REMOVE; }
        g_byte_array_append(input, chunk, (guint)count);
        processed += (size_t)count;
        if (processed >= 256 * 1024) break;
    }
    if (condition & (G_IO_ERR | G_IO_NVAL)) { fail(); return G_SOURCE_REMOVE; }
    return consume_commands();
}

static gboolean tick(gpointer unused) {
    (void)unused;
    if (!consume_commands()) return G_SOURCE_REMOVE;
    for (unsigned index = 0; index < PROP_LIMIT && !failed; index++) {
        if (props[index].window) update_geometry(&props[index]);
    }
    return !failed;
}

static gboolean terminate(gpointer unused) {
    (void)unused;
    stopping = TRUE;
    g_main_loop_quit(loop);
    return G_SOURCE_REMOVE;
}

// Fontconfig needs a file identity. Anonymous, close-on-exec memory files keep
// the already-embedded faces available without writing fonts into user folders.
static int register_font(const unsigned char *bytes, size_t length) {
    int fd = memfd_create("honk-prop-font", MFD_CLOEXEC);
    if (fd < 0) return -1;
    size_t offset = 0;
    while (offset < length) {
        ssize_t count = write(fd, bytes + offset, length - offset);
        if (count < 0 && errno == EINTR) continue;
        if (count <= 0) { close(fd); return -1; }
        offset += (size_t)count;
    }
    char path[64];
    g_snprintf(path, sizeof(path), "/proc/self/fd/%d", fd);
    if (!FcConfigAppFontAddFile(NULL, (const FcChar8 *)path)) { close(fd); return -1; }
    return fd;
}

int honk_props_run(const unsigned char *light, size_t light_len, const unsigned char *bold, size_t bold_len) {
    if (!gtk_init_check()) return 1;
    int light_fd = register_font(light, light_len);
    int bold_fd = register_font(bold, bold_len);
    if (light_fd < 0 || bold_fd < 0) {
        if (light_fd >= 0) close(light_fd);
        if (bold_fd >= 0) close(bold_fd);
        return 1;
    }
    signal(SIGPIPE, SIG_IGN);
    if (fcntl(STDIN_FILENO, F_SETFL, fcntl(STDIN_FILENO, F_GETFL) | O_NONBLOCK) < 0 ||
        fcntl(STDOUT_FILENO, F_SETFL, fcntl(STDOUT_FILENO, F_GETFL) | O_NONBLOCK) < 0) return 1;
    GdkDisplay *display = gdk_display_get_default();
    positioning = GDK_IS_X11_DISPLAY(display);
    GtkCssProvider *css = gtk_css_provider_new();
    gtk_css_provider_load_from_data(css,
        "textview { font-family: 'Makira Light'; font-size: 15px; font-weight: 300; }"
        ".honk-prop-header label { font-family: 'Makira'; font-size: 15px; font-weight: 700; }"
        ".honk-prop-close { min-width: 26px; min-height: 26px; padding: 0; margin: 0; border-radius: 0; }", -1);
    gtk_style_context_add_provider_for_display(display, GTK_STYLE_PROVIDER(css), GTK_STYLE_PROVIDER_PRIORITY_APPLICATION);
    g_object_unref(css);
    input = g_byte_array_sized_new(65536);
    loop = g_main_loop_new(NULL, FALSE);
    g_unix_fd_add(STDIN_FILENO, G_IO_IN | G_IO_HUP | G_IO_ERR | G_IO_NVAL, read_commands, NULL);
    g_unix_signal_add(SIGTERM, terminate, NULL);
    g_unix_signal_add(SIGINT, terminate, NULL);
    g_timeout_add(16, tick, NULL);
    emit(positioning ? "{\"v\":1,\"event\":\"ready\",\"positioning\":true}\n" :
                       "{\"v\":1,\"event\":\"ready\",\"positioning\":false}\n");
    if (!failed) g_main_loop_run(loop);
    stopping = TRUE;
    for (unsigned index = 0; index < PROP_LIMIT; index++) close_prop(&props[index], FALSE);
    g_main_loop_unref(loop);
    g_byte_array_unref(input);
    close(light_fd);
    close(bold_fd);
    return failed ? 1 : 0;
}
