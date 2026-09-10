const std = @import("std");
const runner = @import("runner");
const native_sdk = @import("native_sdk");
const owned_props = @import("owned_props.zig");

test {
    _ = owned_props;
}

pub const panic = std.debug.FullPanic(native_sdk.debug.capturePanic);

const canvas = native_sdk.canvas;
const geometry = native_sdk.geometry;

const canvas_label = "main-canvas";
const window_width: f32 = 860;
const window_height: f32 = 700;

const app_permissions = [_][]const u8{ native_sdk.security.permission_command, native_sdk.security.permission_view };
const shell_views = [_]native_sdk.ShellView{
    .{ .label = canvas_label, .kind = .gpu_surface, .fill = true, .role = "Goose", .accessibility_label = "Goose", .gpu_backend = .metal, .gpu_pixel_format = .bgra8_unorm, .gpu_present_mode = .timer, .gpu_alpha_mode = .@"opaque", .gpu_color_space = .srgb, .gpu_vsync = true },
};
const shell_windows = [_]native_sdk.ShellWindow{.{
    .label = "main",
    .title = "Goose",
    .width = window_width,
    .height = window_height,
    .restore_state = false,
    .views = &shell_views,
}};
const shell_scene: native_sdk.ShellConfig = .{ .windows = &shell_windows };

const dev = @import("builtin").mode == .Debug;
pub const Effects = native_sdk.Effects(Msg);
pub const AppUi = canvas.Ui(Msg);
pub const app_markup = @embedFile("app.native");
pub const version = "1.11.1";
const SettingsApp = native_sdk.UiAppWithFeatures(Model, Msg, .{ .runtime_markup = dev });
const CompiledView = canvas.CompiledMarkupView(Model, Msg, app_markup);
const body_font: canvas.FontId = canvas.min_registered_font_id;
const heading_font: canvas.FontId = body_font + 1;
const fonts = [_]SettingsApp.FontRegistration{
    .{ .id = body_font, .name = "Makira Light", .ttf = @embedFile("fonts/Makira-Light.ttf") },
    .{ .id = heading_font, .name = "Makira Bold", .ttf = @embedFile("fonts/Makira-Bold.ttf") },
};
const Buffer = canvas.TextBuffer;
const Page = enum { general, appearance, behavior, sound, platform };
const Action = enum { read, save, validate, status, start, stop, check_updates, update, kde_setup, kde_remove, sway_setup, sway_remove, hyprland_setup, hyprland_remove, gnome_setup, gnome_remove, pointer_request, pointer_cancel };

pub const Field = struct {
    index: usize = 0,
    key_buffer: Buffer(80) = .{},
    label_buffer: Buffer(96) = .{},
    help_buffer: Buffer(192) = .{},
    value_buffer: Buffer(64) = .{},
    original: Buffer(64) = .{},
    kind: enum { toggle, number, color, text } = .text,
    page: Page = .general,
    nullable: bool = false,
    pub fn key(f: *const Field) []const u8 {
        return f.key_buffer.text();
    }
    pub fn label(f: *const Field) []const u8 {
        return f.label_buffer.text();
    }
    pub fn hasHelp(f: *const Field) bool {
        return f.help_buffer.len > 0;
    }
    pub fn help(f: *const Field) []const u8 {
        return f.help_buffer.text();
    }
    pub fn value(f: *const Field) []const u8 {
        return f.value_buffer.text();
    }
    pub fn isToggle(f: *const Field) bool {
        return f.kind == .toggle;
    }
    pub fn checked(f: *const Field) bool {
        return std.mem.eql(u8, f.value(), "true");
    }
    pub fn changed(f: *const Field) bool {
        return !std.mem.eql(u8, f.value(), f.original.text());
    }
};

pub const Msg = union(enum) {
    general,
    appearance,
    behavior,
    sound,
    platform,
    toggle: usize,
    edit: usize,
    edit_input: canvas.TextInputEvent,
    edit_done,
    edit_cancel,
    save,
    reload,
    discard,
    keep_draft,
    check_updates,
    update_now,
    refresh,
    start,
    stop,
    kde_setup,
    kde_confirm,
    kde_cancel,
    kde_remove,
    sway_setup,
    sway_confirm,
    sway_cancel,
    sway_remove,
    hyprland_setup,
    hyprland_confirm,
    hyprland_cancel,
    hyprland_remove,
    gnome_setup,
    gnome_confirm,
    gnome_cancel,
    gnome_remove,
    pointer_request,
    pointer_cancel,
    completed: native_sdk.EffectExit,
    system_appearance: native_sdk.Appearance,
    pub const view_unbound = .{ "completed", "system_appearance" };
};

pub const Model = struct {
    // Used by derived view methods or the stdio lifecycle, never bound directly.
    pub const view_unbound = .{ "fields", "field_count", "page", "service", "config_path", "revision", "version", "status_buffer", "runtime_buffer", "update_buffer", "integration_buffer", "sway_buffer", "hyprland_buffer", "gnome_buffer", "pointer_buffer", "pointer_request_available", "pointer_cancel_available", "loaded", "request_id", "editing", "edit_buffer", "update_available", "update_managed", "system_appearance" };
    system_appearance: native_sdk.Appearance = .{},
    fields: [64]Field = @splat(.{}),
    field_count: usize = 0,
    page: Page = .general,
    service: Buffer(2048) = .{},
    config_path: Buffer(2048) = .{},
    revision: Buffer(80) = .{},
    version: Buffer(32) = .{},
    status_buffer: Buffer(2048) = Buffer(2048).init("Loading settings..."),
    runtime_buffer: Buffer(2048) = .{},
    update_buffer: Buffer(1024) = Buffer(1024).init("Check for a newer stable release when you are ready."),
    busy: bool = false,
    loaded: bool = false,
    request_id: u64 = 0,
    editing: ?usize = null,
    edit_buffer: Buffer(64) = .{},
    discard_prompt: bool = false,
    update_available: bool = false,
    update_managed: bool = false,
    integration_buffer: Buffer(2048) = .{},
    integration_supported: bool = false,
    kde_installed: bool = false,
    kde_prompt: bool = false,
    sway_buffer: Buffer(2048) = .{},
    sway_supported: bool = false,
    sway_installed: bool = false,
    sway_prompt: bool = false,
    hyprland_buffer: Buffer(2048) = .{},
    hyprland_supported: bool = false,
    hyprland_installed: bool = false,
    hyprland_prompt: bool = false,
    gnome_buffer: Buffer(2048) = .{},
    gnome_supported: bool = false,
    gnome_installed: bool = false,
    gnome_prompt: bool = false,
    pointer_buffer: Buffer(1024) = .{},
    pointer_request_available: bool = false,
    pointer_cancel_available: bool = false,

    pub fn pointerStatus(m: *const Model) []const u8 {
        return m.pointer_buffer.text();
    }
    pub fn canRequestPointer(m: *const Model) bool {
        return m.pointer_request_available and !m.busy;
    }
    pub fn canCancelPointer(m: *const Model) bool {
        return m.pointer_cancel_available and !m.busy;
    }

    pub fn integrationStatus(m: *const Model) []const u8 {
        return m.integration_buffer.text();
    }
    pub fn swayStatus(m: *const Model) []const u8 {
        return m.sway_buffer.text();
    }
    pub fn canSetupSway(m: *const Model) bool {
        return m.sway_supported and !m.busy;
    }
    pub fn hyprlandStatus(m: *const Model) []const u8 {
        return m.hyprland_buffer.text();
    }
    pub fn canSetupHyprland(m: *const Model) bool {
        return m.hyprland_supported and !m.busy;
    }
    pub fn gnomeStatus(m: *const Model) []const u8 {
        return m.gnome_buffer.text();
    }
    pub fn canSetupGnome(m: *const Model) bool {
        return m.gnome_supported and !m.busy;
    }
    pub fn canSetupKde(m: *const Model) bool {
        return m.integration_supported and !m.busy;
    }

    pub fn status(m: *const Model) []const u8 {
        return m.status_buffer.text();
    }
    pub fn runtime(m: *const Model) []const u8 {
        return m.runtime_buffer.text();
    }
    pub fn updateStatus(m: *const Model) []const u8 {
        return m.update_buffer.text();
    }
    pub fn versionText(m: *const Model) []const u8 {
        return m.version.text();
    }
    pub fn hasEditor(m: *const Model) bool {
        return m.editing != null;
    }
    pub fn editText(m: *const Model) []const u8 {
        return m.edit_buffer.text();
    }
    pub fn editLabel(m: *const Model) []const u8 {
        return if (m.editing) |i| m.fields[i].label() else "";
    }
    pub fn editHelp(m: *const Model) []const u8 {
        return if (m.editing) |i| m.fields[i].help() else "";
    }
    pub fn isPlatform(m: *const Model) bool {
        return m.page == .platform;
    }
    pub fn isGeneral(m: *const Model) bool {
        return m.page == .general;
    }
    pub fn canEdit(m: *const Model) bool {
        return m.loaded and !m.busy;
    }
    pub fn dirty(m: *const Model) bool {
        for (m.fields[0..m.field_count]) |*f| {
            if (f.changed()) return true;
        }
        return false;
    }
    pub fn canSave(m: *const Model) bool {
        return m.canEdit() and m.dirty();
    }
    pub fn canUpdate(m: *const Model) bool {
        return !m.busy and m.update_available and m.update_managed;
    }
    pub fn heading(m: *const Model) []const u8 {
        return switch (m.page) {
            .general => "General",
            .appearance => "Appearance",
            .behavior => "Behavior",
            .sound => "Sound & manners",
            .platform => "Platform & status",
        };
    }
    pub fn visible(m: *const Model, arena: std.mem.Allocator) []const Field {
        const rows = arena.alloc(Field, m.field_count) catch return &.{};
        var count: usize = 0;
        for (m.fields[0..m.field_count]) |f| {
            // Also protect older service responses that list this Linux-only field.
            if (@import("builtin").os.tag != .linux and std.mem.eql(u8, f.key(), "platform.wayland")) continue;
            if (f.page == m.page) {
                rows[count] = f;
                count += 1;
            }
        }
        return rows[0..count];
    }

    pub fn generalTabVariant(m: *const Model) []const u8 {
        return m.tabVariant(.general);
    }
    pub fn appearanceTabVariant(m: *const Model) []const u8 {
        return m.tabVariant(.appearance);
    }
    pub fn behaviorTabVariant(m: *const Model) []const u8 {
        return m.tabVariant(.behavior);
    }
    pub fn soundTabVariant(m: *const Model) []const u8 {
        return m.tabVariant(.sound);
    }
    pub fn platformTabVariant(m: *const Model) []const u8 {
        return m.tabVariant(.platform);
    }
    fn tabVariant(m: *const Model, page: Page) []const u8 {
        return if (m.page == page) "secondary" else "ghost";
    }
};

fn boot(model: *Model, fx: *Effects) void {
    submit(model, .read, fx);
}

pub fn update(model: *Model, msg: Msg, fx: *Effects) void {
    switch (msg) {
        .system_appearance => |appearance| model.system_appearance = appearance,
        .general => model.page = .general,
        .appearance => model.page = .appearance,
        .behavior => model.page = .behavior,
        .sound => model.page = .sound,
        .platform => model.page = .platform,
        .toggle => |i| if (model.canEdit() and i < model.field_count and model.fields[i].isToggle()) {
            model.fields[i].value_buffer.set(if (model.fields[i].checked()) "false" else "true");
        },
        .edit => |i| if (model.canEdit() and i < model.field_count) {
            model.editing = i;
            model.edit_buffer.set(model.fields[i].value());
        },
        .edit_input => |edit| model.edit_buffer.apply(edit),
        .edit_cancel => model.editing = null,
        .edit_done => if (model.editing) |i| {
            if (model.edit_buffer.truncated) {
                model.status_buffer.set("Value is too long. Shorten it before applying.");
                return;
            }
            model.fields[i].value_buffer.set(model.edit_buffer.text());
            model.editing = null;
        },
        .reload => if (!model.busy) {
            if (model.dirty()) model.discard_prompt = true else submit(model, .read, fx);
        },
        .discard => {
            model.discard_prompt = false;
            submit(model, .read, fx);
        },
        .keep_draft => model.discard_prompt = false,
        .save => if (model.canSave()) {
            submit(model, .save, fx);
        },
        .refresh => submit(model, .status, fx),
        .kde_setup => if (model.canSetupKde()) {
            model.kde_prompt = true;
        },
        .kde_cancel => model.kde_prompt = false,
        .kde_confirm => if (model.canSetupKde()) {
            model.kde_prompt = false;
            submit(model, .kde_setup, fx);
        },
        .kde_remove => if (model.canSetupKde()) {
            submit(model, .kde_remove, fx);
        },
        .sway_setup => if (model.canSetupSway()) {
            model.sway_prompt = true;
        },
        .sway_cancel => model.sway_prompt = false,
        .sway_confirm => if (model.canSetupSway()) {
            model.sway_prompt = false;
            submit(model, .sway_setup, fx);
        },
        .sway_remove => if (model.canSetupSway()) {
            submit(model, .sway_remove, fx);
        },
        .hyprland_setup => if (model.canSetupHyprland()) {
            model.hyprland_prompt = true;
        },
        .hyprland_cancel => model.hyprland_prompt = false,
        .hyprland_confirm => if (model.canSetupHyprland()) {
            model.hyprland_prompt = false;
            submit(model, .hyprland_setup, fx);
        },
        .hyprland_remove => if (model.canSetupHyprland()) {
            submit(model, .hyprland_remove, fx);
        },
        .gnome_setup => if (model.canSetupGnome()) {
            model.gnome_prompt = true;
        },
        .gnome_cancel => model.gnome_prompt = false,
        .gnome_confirm => if (model.canSetupGnome()) {
            model.gnome_prompt = false;
            submit(model, .gnome_setup, fx);
        },
        .gnome_remove => if (model.canSetupGnome()) {
            submit(model, .gnome_remove, fx);
        },
        .pointer_request => if (model.canRequestPointer()) {
            submit(model, .pointer_request, fx);
        },
        .pointer_cancel => if (model.canCancelPointer()) {
            submit(model, .pointer_cancel, fx);
        },
        .check_updates => submit(model, .check_updates, fx),
        .update_now => if (model.canUpdate()) {
            submit(model, .update, fx);
        },
        .start => if (model.dirty()) {
            model.status_buffer.set("Save your changes before starting the goose.");
        } else {
            submit(model, .start, fx);
        },
        .stop => submit(model, .stop, fx),
        .completed => |exit| {
            model.busy = false;
            if (exit.reason != .exited or exit.code != 0 or exit.output_truncated) {
                model.status_buffer.set(if (exit.stderr_tail.len > 0) exit.stderr_tail else "The settings service did not complete. Reload to retry.");
                return;
            }
            acceptResponse(model, exit.output) catch |err| {
                var buffer: [256]u8 = undefined;
                model.status_buffer.set(std.fmt.bufPrint(&buffer, "Cannot read settings response: {s}", .{@errorName(err)}) catch "Invalid settings response.");
            };
        },
    }
}

fn submit(model: *Model, action: Action, fx: *Effects) void {
    if (model.busy) return;
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    if (action == .check_updates) {
        model.update_available = false;
        model.update_managed = false;
    }
    model.request_id += 1;
    const input = requestJson(model, action, arena.allocator()) catch {
        model.status_buffer.set("One or more values are invalid. Numbers must be finite; colors use #RRGGBB.");
        return;
    };
    if (input.len > 4096) {
        model.status_buffer.set("Too many changes for one save. Save a smaller group first.");
        return;
    }
    model.busy = true;
    model.status_buffer.set(if (action == .check_updates) "Checking for updates..." else "Working...");
    const basic = [_][]const u8{ model.service.text(), "__settings-service" };
    const with_path = [_][]const u8{ model.service.text(), "__settings-service", "--config", model.config_path.text() };
    fx.spawn(.{ .key = 1, .argv = if (model.config_path.len == 0) &basic else &with_path, .stdin = input, .output = .collect, .on_exit = Effects.exitMsg(.completed) });
}

fn requestJson(model: *const Model, action: Action, allocator: std.mem.Allocator) ![]const u8 {
    var command = std.json.ObjectMap{};
    try command.put(allocator, "op", .{ .string = @tagName(action) });
    if (action == .save or action == .validate) {
        try command.put(allocator, "revision", .{ .string = model.revision.text() });
        var patch = std.json.ObjectMap{};
        for (model.fields[0..model.field_count]) |*f| {
            if (!f.changed()) continue;
            const value: std.json.Value = switch (f.kind) {
                .toggle => .{ .bool = f.checked() },
                .number => blk: {
                    const number = try std.fmt.parseFloat(f64, f.value());
                    if (!std.math.isFinite(number)) return error.InvalidNumber;
                    break :blk .{ .float = number };
                },
                .color => if (f.nullable and f.value().len == 0) .null else .{ .string = f.value() },
                .text => .{ .string = f.value() },
            };
            try patch.put(allocator, f.key(), value);
        }
        try command.put(allocator, "patch", .{ .object = patch });
    }
    return std.json.Stringify.valueAlloc(allocator, .{ .protocol = 1, .request_id = model.request_id, .command = std.json.Value{ .object = command } }, .{});
}

fn string(value: std.json.Value, key: []const u8) []const u8 {
    if (value != .object) return "";
    const item = value.object.get(key) orelse return "";
    return if (item == .string) item.string else "";
}
fn flag(value: std.json.Value, key: []const u8) bool {
    if (value != .object) return false;
    const item = value.object.get(key) orelse return false;
    return item == .bool and item.bool;
}

pub fn acceptResponse(model: *Model, bytes: []const u8) !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();
    const parsed = try std.json.parseFromSlice(std.json.Value, allocator, bytes, .{});
    const root = parsed.value;
    if (root != .object or root.object.get("protocol") == null or root.object.get("request_id") == null) return error.InvalidEnvelope;
    const protocol = root.object.get("protocol").?;
    const id = root.object.get("request_id").?;
    if (protocol != .integer or protocol.integer != 1 or id != .integer or id.integer < 0 or @as(u64, @intCast(id.integer)) != model.request_id) return error.WrongEnvelope;
    if (!flag(root, "ok")) {
        const failure = root.object.get("error") orelse return error.MissingError;
        model.status_buffer.set(string(failure, "message"));
        return;
    }
    const data = root.object.get("data") orelse return error.MissingData;
    if (data != .object) return error.InvalidData;
    if (data.object.get("fields")) |fields| {
        if (fields != .array or fields.array.items.len > model.fields.len) return error.TooManyFields;
        // Parse into a separate draft so a malformed reply never partially replaces edits.
        const next = try allocator.create([64]Field);
        next.* = @splat(.{});
        for (fields.array.items, 0..) |field, i| {
            var f = &next[i];
            f.index = i;
            try copyBounded(&f.key_buffer, string(field, "key"));
            try copyBounded(&f.label_buffer, string(field, "label"));
            try copyBounded(&f.help_buffer, string(field, "help"));
            f.kind = std.meta.stringToEnum(@TypeOf(f.kind), string(field, "kind")) orelse return error.UnknownKind;
            const page = string(field, "page");
            f.page = if (std.mem.eql(u8, page, "Appearance")) .appearance else if (std.mem.eql(u8, page, "Behavior")) .behavior else if (std.mem.eql(u8, page, "Sound & manners")) .sound else if (std.mem.eql(u8, page, "Platform & status")) .platform else .general;
            f.nullable = std.mem.eql(u8, f.key(), "colors.goose_shade") or std.mem.eql(u8, f.key(), "colors.goose_wing") or std.mem.eql(u8, f.key(), "colors.goose_orange_dark");
            const value = field.object.get("value") orelse return error.MissingValue;
            const display = if (value == .string) value.string else if (value == .null) "" else try std.json.Stringify.valueAlloc(allocator, value, .{});
            try copyBounded(&f.value_buffer, display);
            f.original.set(display);
        }
        try copyBounded(&model.revision, string(data, "revision"));
        model.fields = next.*;
        model.field_count = fields.array.items.len;
        model.version.set(string(data, "version"));
        model.loaded = true;
    }
    if (data.object.get("runtime")) |runtime| {
        const runtime_error = string(runtime, "error");
        const description = if (runtime_error.len > 0)
            try std.fmt.allocPrint(allocator, "Goose status unavailable\n{s}\nSettings remain editable. Refresh to try again.", .{runtime_error})
        else
            try std.fmt.allocPrint(allocator, "Goose: {s}\nDesktop: {s}\nOverlay: {s}  |  Sound: {s}\nCursor: {s}  |  Window rides: {s}\nNotes and memes: {s}\nFullscreen observation: {s}  |  Do not disturb: {s}\nAccessibility: {s}", .{
                if (flag(runtime, "running")) "running" else "stopped", string(runtime, "platform"), string(runtime, "overlay"), string(runtime, "audio"), string(runtime, "cursor"), string(runtime, "windows"), string(runtime, "notes_and_memes"), if (string(runtime, "fullscreen").len > 0) string(runtime, "fullscreen") else "unprobed", if (string(runtime, "dnd").len > 0) string(runtime, "dnd") else "unprobed", string(runtime, "accessibility"),
            });
        if (runtime.object.get("session")) |session| {
            if (session == .object) {
                model.runtime_buffer.set(try std.fmt.allocPrint(allocator, "{s}\nDisplay: {s}\nSession reports: {s}\nNote placement: {s}", .{
                    description, string(session, "backend"), string(session, "desktop_hint"), string(session, "prop_positioning"),
                }));
            } else model.runtime_buffer.set(description);
        } else model.runtime_buffer.set(description);
    }
    if (data.object.get("updates")) |updates| {
        model.update_available = flag(updates, "available");
        model.update_managed = flag(updates, "managed");
        model.update_buffer.set(string(updates, "message"));
    }
    if (data.object.get("integrations")) |integrations| {
        model.integration_supported = flag(integrations, "supported");
        model.kde_installed = flag(integrations, "installed");
        model.pointer_request_available = false;
        model.pointer_cancel_available = false;
        model.pointer_buffer.set("");
        const detail = string(integrations, "description");
        if (integrations == .object) {
            if (integrations.object.get("pointer")) |pointer| {
                model.pointer_request_available = flag(pointer, "can_request");
                model.pointer_cancel_available = flag(pointer, "can_cancel");
                model.pointer_buffer.set(string(pointer, "description"));
            }
            if (integrations.object.get("capabilities")) |caps| {
                if (caps == .object) {
                    model.integration_buffer.set(try std.fmt.allocPrint(allocator, "{s}\nWindow observation: {s} | Movement: {s}\nPointer observation: {s} | Control: {s}\nFullscreen: {s} | Do not disturb: {s}\nAnimated prop placement: {s}", .{
                        detail,                          string(caps, "windows"),    string(caps, "movement"), string(caps, "pointer_observation"),
                        string(caps, "pointer_control"), string(caps, "fullscreen"), string(caps, "dnd"),      string(caps, "prop_positioning"),
                    }));
                } else model.integration_buffer.set(detail);
            } else model.integration_buffer.set(detail);
        }
    }
    if (data.object.get("sway")) |sway| {
        model.sway_supported = flag(sway, "supported");
        model.sway_installed = flag(sway, "installed");
        const detail = string(sway, "description");
        model.sway_buffer.set(detail);
        if (sway == .object) {
            if (sway.object.get("capabilities")) |caps| {
                if (caps == .object) {
                    model.sway_buffer.set(try std.fmt.allocPrint(allocator, "{s}\nWindow observation: {s} | Fullscreen: {s}\nMovement: {s} | Pointer control: {s}\nOwned notes use normal desktop placement.", .{
                        detail, string(caps, "windows"), string(caps, "fullscreen"), string(caps, "movement"), string(caps, "pointer_control"),
                    }));
                }
            }
        }
    }
    if (data.object.get("hyprland")) |hyprland| {
        model.hyprland_supported = flag(hyprland, "supported");
        model.hyprland_installed = flag(hyprland, "installed");
        const detail = string(hyprland, "description");
        model.hyprland_buffer.set(detail);
        if (hyprland == .object) {
            if (hyprland.object.get("capabilities")) |caps| {
                if (caps == .object) {
                    model.hyprland_buffer.set(try std.fmt.allocPrint(allocator, "{s}\nWindow observation: {s} | Fullscreen: {s}\nMovement: {s} | Pointer control: {s}\nOwned notes use normal desktop placement.", .{
                        detail, string(caps, "windows"), string(caps, "fullscreen"), string(caps, "movement"), string(caps, "pointer_control"),
                    }));
                }
            }
        }
    }
    if (data.object.get("gnome")) |gnome| {
        model.gnome_supported = flag(gnome, "supported");
        model.gnome_installed = flag(gnome, "installed");
        const detail = string(gnome, "description");
        model.gnome_buffer.set(detail);
        if (gnome == .object) {
            if (gnome.object.get("capabilities")) |caps| {
                if (caps == .object) {
                    model.gnome_buffer.set(try std.fmt.allocPrint(allocator, "{s}\nWindow and user-drag observations: {s} | Fullscreen: {s}\nMovement: {s} | Pointer control: {s}\nOwned-note positioning: {s}.", .{
                        detail, string(caps, "windows"), string(caps, "fullscreen"), string(caps, "movement"), string(caps, "pointer_control"), string(caps, "prop_positioning"),
                    }));
                }
            }
        }
    }
    const message = string(data, "message");
    const warning = string(data, "warning");
    const restart = data.object.get("restart_required");
    model.status_buffer.set(try std.fmt.allocPrint(allocator, "{s}{s}{s}{s}", .{
        if (message.len > 0) message else "Ready.",                                                                                                 if (warning.len > 0) " Warning: " else "", warning,
        if (restart != null and restart.? == .array and restart.?.array.items.len > 0) " Restart required for the desktop session change." else "",
    }));
}
fn copyBounded(buffer: anytype, value: []const u8) !void {
    if (value.len > buffer.storage.len) return error.ValueTooLong;
    buffer.set(value);
}

fn appearanceChanged(appearance: native_sdk.Appearance) ?Msg {
    return .{ .system_appearance = appearance };
}

pub fn designTokens(model: *const Model) canvas.DesignTokens {
    const appearance = model.system_appearance;
    const scheme: canvas.ColorScheme = if (appearance.color_scheme == .dark) .dark else .light;
    var tokens = canvas.DesignTokens.theme(.{
        .color_scheme = scheme,
        .contrast = if (appearance.high_contrast) .high else .standard,
        .reduce_motion = appearance.reduce_motion,
    });
    if (!appearance.high_contrast) {
        tokens = tokens.withOverrides(canvas.accentOverrides(canvas.Color.rgb8(252, 121, 39), scheme));
    }
    tokens.typography.font_id = body_font;
    tokens.typography.bold_font_id = heading_font;
    tokens.typography.button_font_id = body_font;
    // A Windows notch arrives as 40 logical pixels. Start with a small step,
    // then settle the short tail in about a quarter second. SDK defaults decay
    // by only 14% PER SECOND, which sends a single notch to the page boundary.
    tokens.scroll = .{
        .wheel_multiplier = if (appearance.reduce_motion) 1 else 0.25,
        .wheel_velocity_scale = if (appearance.reduce_motion) 0 else 80,
        .deceleration_per_second = 0.00000001523, // exp(-18), independent of frame rate.
        .stop_velocity = 8,
    };
    return tokens;
}

pub fn main(init: std.process.Init) !void {
    const args = try init.minimal.args.toSlice(std.heap.page_allocator);
    defer std.heap.page_allocator.free(args);
    if (args.len == 2 and std.mem.eql(u8, args[1], "--version")) {
        std.debug.print("honk300-settings {s}\n", .{version});
        return;
    }
    if (@import("builtin").os.tag == .linux and args.len == 2 and std.mem.eql(u8, args[1], "--owned-props")) {
        comptime {
            if (@import("builtin").os.tag == .linux) @export(&owned_props.decode, .{ .name = "honk_props_decode" });
        }
        return owned_props.run();
    }
    // The app struct (and any real Model) is multi-MB: `create`
    // heap-allocates and constructs everything in place, so neither
    // ever rides the stack. Mutate `app_state.model` through the
    // pointer before running if boot state is not the default.
    const app_state = try SettingsApp.create(std.heap.page_allocator, .{
        .name = "honk300-settings",
        .scene = shell_scene,
        .canvas_label = canvas_label,
        .update_fx = update,
        .init_fx = boot,
        .tokens_fn = designTokens,
        .on_appearance = appearanceChanged,
        .fonts = &fonts,
        .view = CompiledView.build,
        .markup = if (dev) .{ .source = app_markup, .watch_path = "src/app.native", .io = init.io } else null,
    });
    defer app_state.destroy();
    const executable = try std.process.executablePathAlloc(init.io, std.heap.page_allocator);
    defer std.heap.page_allocator.free(executable);
    const parent = std.fs.path.dirname(executable) orelse return error.MissingExecutableDirectory;
    const service_name = if (@import("builtin").os.tag == .windows) "honk300.exe" else "honk300";
    const service = try std.fs.path.join(std.heap.page_allocator, &.{ parent, service_name });
    defer std.heap.page_allocator.free(service);
    if (service.len > 2048) return error.ExecutablePathTooLong;
    app_state.model.service.set(service);
    if (args.len > 1) {
        if (args.len != 3 or !std.mem.eql(u8, args[1], "--config") or args[2].len > 2048) return error.InvalidArguments;
        app_state.model.config_path.set(args[2]);
    }

    try runner.runWithOptions(app_state.app(), .{
        .app_name = "honk300-settings",
        .window_title = "Goose",
        .bundle_id = "dev.emmetts.honk300.settings",
        .icon_path = "assets/icon.png",
        .default_frame = geometry.RectF.init(0, 0, window_width, window_height),
        .restore_state = false,
        .js_window_api = false,
        .security = .{
            .permissions = &app_permissions,
            .navigation = .{ .allowed_origins = &.{ "zero://inline", "zero://app" } },
        },
    }, init);
}

test {
    _ = @import("tests.zig");
}
