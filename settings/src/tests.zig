const std = @import("std");
const main = @import("main.zig");
const sdk = @import("native_sdk");

test "native accessibility activates an offscreen button through actual scrolling and keyboard routing" {
    const App = struct {
        activations: usize = 0,
        fn app(self: *@This()) sdk.App {
            return .{ .context = self, .name = "Honk300 accessibility regression", .source = sdk.platform.WebViewSource.html("<h1>Settings</h1>"), .event_fn = event };
        }
        fn event(context: *anyopaque, _: *sdk.Runtime, value: sdk.Event) !void {
            const self: *@This() = @ptrCast(@alignCast(context));
            switch (value) {
                .canvas_widget_keyboard => |keyboard| if (keyboard.target) |target| {
                    if (target.id == 4) self.activations += 1;
                },
                else => {},
            }
        }
    };
    const harness = try sdk.TestHarness().create(std.testing.allocator, .{});
    defer harness.destroy(std.testing.allocator);
    harness.null_platform.gpu_surfaces = true;
    var state = App{};
    try harness.start(state.app());
    _ = try harness.runtime.createView(.{ .window_id = 1, .label = "canvas", .kind = .gpu_surface, .frame = sdk.geometry.RectF.init(0, 0, 320, 200) });
    const items = [_]sdk.canvas.Widget{
        .{ .id = 3, .kind = .button, .text = "Visible", .frame = sdk.geometry.RectF.init(0, 0, 250, 32) },
        .{ .id = 4, .kind = .button, .text = "Request pointer access", .frame = sdk.geometry.RectF.init(0, 280, 250, 32) },
        .{ .id = 5, .kind = .button, .text = "Disabled", .frame = sdk.geometry.RectF.init(0, 350, 250, 32), .state = .{ .disabled = true } },
    };
    const children = [_]sdk.canvas.Widget{.{ .id = 2, .kind = .scroll_view, .frame = sdk.geometry.RectF.init(12, 12, 280, 90), .children = &items }};
    var nodes: [8]sdk.canvas.WidgetLayoutNode = undefined;
    const layout = try sdk.canvas.layoutWidgetTree(.{ .id = 1, .kind = .panel, .children = &children }, sdk.geometry.RectF.init(0, 0, 320, 200), &nodes);
    _ = try harness.runtime.setCanvasWidgetLayout(1, "canvas", layout);
    try std.testing.expect(layout.focusTargetById(4) == null);
    _ = try harness.runtime.dispatchCanvasWidgetAccessibilityAction(state.app(), 1, "canvas", .{ .id = 4, .action = .press });
    try std.testing.expectEqual(@as(usize, 1), state.activations);
    const scrolled = try harness.runtime.canvasWidgetLayout(1, "canvas");
    try std.testing.expect(scrolled.findById(2).?.widget.value > 0);
    try std.testing.expect(scrolled.focusTargetById(4) != null);
    const offset = scrolled.findById(2).?.widget.value;
    try std.testing.expectError(error.InvalidCommand, harness.runtime.dispatchCanvasWidgetAccessibilityAction(state.app(), 1, "canvas", .{ .id = 5, .action = .press }));
    const unchanged = try harness.runtime.canvasWidgetLayout(1, "canvas");
    try std.testing.expectEqual(offset, unchanged.findById(2).?.widget.value);
    try std.testing.expectEqual(@as(usize, 1), state.activations);
}

test "all settings pages and dialogs build against a real Rust service response" {
    var arena = std.heap.ArenaAllocator.init(std.testing.allocator);
    defer arena.deinit();
    const model = try arena.allocator().create(main.Model);
    model.* = .{};
    model.request_id = 1;
    try main.acceptResponse(model, @embedFile("fixtures/read.json"));
    try std.testing.expectEqual(@as(usize, 53), model.field_count);
    const View = sdk.canvas.CompiledMarkupView(main.Model, main.Msg, main.app_markup);
    inline for (std.meta.tags(@TypeOf(model.page))) |page| {
        model.page = page;
        var ui = main.AppUi.init(arena.allocator());
        _ = try ui.finalize(View.build(&ui, model));
    }
    model.editing = 1;
    var ui = main.AppUi.init(arena.allocator());
    _ = try ui.finalize(View.build(&ui, model));
    model.editing = null;
    model.discard_prompt = true;
    ui = main.AppUi.init(arena.allocator());
    _ = try ui.finalize(View.build(&ui, model));
}

test "a conflict preserves the user's draft until explicit discard" {
    var model = main.Model{};
    model.request_id = 1;
    try main.acceptResponse(&model, @embedFile("fixtures/read.json"));
    model.fields[0].value_buffer.set("true");
    try main.acceptResponse(&model,
        \\{"protocol":1,"request_id":1,"ok":false,"error":{"code":"conflict","message":"Reload before saving"}}
    );
    try std.testing.expect(model.dirty());
    try std.testing.expectEqualStrings("true", model.fields[0].value());
}

test "invalid reply cannot replace unsaved fields" {
    var model = main.Model{};
    model.request_id = 3;
    try std.testing.expectError(error.WrongEnvelope, main.acceptResponse(&model,
        \\{"protocol":2,"request_id":3,"ok":true,"data":{}}
    ));
    try std.testing.expect(!model.loaded);
}

test "unknown runtime status preserves editable settings without claiming the goose stopped" {
    var model = main.Model{};
    model.request_id = 1;
    try main.acceptResponse(&model, @embedFile("fixtures/read.json"));
    model.fields[0].value_buffer.set("true");
    try main.acceptResponse(&model,
        \\{"protocol":1,"request_id":1,"ok":true,"data":{"runtime":{"available":false,"running":null,"error":"Runtime status could not be confirmed"}}}
    );
    try std.testing.expect(model.loaded);
    try std.testing.expect(model.dirty());
    try std.testing.expectEqual(@as(usize, 53), model.field_count);
    try std.testing.expect(std.mem.indexOf(u8, model.runtime(), "status unavailable") != null);
    try std.testing.expect(std.mem.indexOf(u8, model.runtime(), "stopped") == null);
}

test "fullscreen and DND status stay independent of legacy manners and saved settings" {
    var model = main.Model{};
    model.request_id = 1;
    try main.acceptResponse(&model, @embedFile("fixtures/read.json"));
    model.fields[0].value_buffer.set("true");
    try main.acceptResponse(&model,
        \\{"protocol":1,"request_id":1,"ok":true,"data":{"runtime":{"running":true,"manners":"supported","fullscreen":"denied","dnd":"unsupported"}}}
    );
    try std.testing.expect(model.dirty());
    try std.testing.expect(std.mem.indexOf(u8, model.runtime(), "Fullscreen observation: denied") != null);
    try std.testing.expect(std.mem.indexOf(u8, model.runtime(), "Do not disturb: unsupported") != null);
    try main.acceptResponse(&model,
        \\{"protocol":1,"request_id":1,"ok":true,"data":{"runtime":{"running":true,"manners":"supported"}}}
    );
    try std.testing.expect(model.dirty());
    try std.testing.expect(std.mem.indexOf(u8, model.runtime(), "Fullscreen observation: unprobed") != null);
    try std.testing.expect(std.mem.indexOf(u8, model.runtime(), "Do not disturb: unprobed") != null);
}

test "KDE permission responses and native consent dialog preserve unsaved settings" {
    var arena = std.heap.ArenaAllocator.init(std.testing.allocator);
    defer arena.deinit();
    const model = try arena.allocator().create(main.Model);
    model.* = .{};
    model.request_id = 1;
    try main.acceptResponse(model, @embedFile("fixtures/read.json"));
    model.fields[0].value_buffer.set("true");
    try main.acceptResponse(model,
        \\{"protocol":1,"request_id":1,"ok":true,"data":{"integrations":{"supported":true,"installed":true,"description":"KDE companion enabled","pointer":{"description":"Waiting for desktop consent","can_request":false,"can_cancel":true},"capabilities":{"windows":"supported","movement":"supported","pointer_observation":"supported","pointer_control":"unprobed","fullscreen":"supported","dnd":"unsupported","prop_positioning":"unsupported"}}}}
    );
    try std.testing.expect(model.dirty());
    try std.testing.expect(model.kde_installed);
    try std.testing.expect(std.mem.indexOf(u8, model.integrationStatus(), "Control: unprobed") != null);
    try std.testing.expect(model.canCancelPointer());
    try std.testing.expect(!model.canRequestPointer());
    model.page = .platform;
    const View = sdk.canvas.CompiledMarkupView(main.Model, main.Msg, main.app_markup);
    var ui = main.AppUi.init(arena.allocator());
    _ = try ui.finalize(View.build(&ui, model));
    model.kde_prompt = true;
    ui = main.AppUi.init(arena.allocator());
    _ = try ui.finalize(View.build(&ui, model));
    try main.acceptResponse(model,
        \\{"protocol":1,"request_id":1,"ok":true,"data":{"integrations":{"supported":true,"installed":false,"description":"KDE integration is off"}}}
    );
    try std.testing.expect(!model.kde_installed);
    try std.testing.expect(!model.canCancelPointer());
    try std.testing.expect(!model.canRequestPointer());
    try std.testing.expectEqualStrings("", model.pointerStatus());
    try std.testing.expect(model.dirty());
    try std.testing.expectEqualStrings("true", model.fields[0].value());
}

test "Sway setup and removal retain the settings draft and separate capability limits" {
    var arena = std.heap.ArenaAllocator.init(std.testing.allocator);
    defer arena.deinit();
    const model = try arena.allocator().create(main.Model);
    model.* = .{};
    model.request_id = 1;
    try main.acceptResponse(model, @embedFile("fixtures/read.json"));
    model.fields[0].value_buffer.set("true");
    try main.acceptResponse(model,
        \\{"protocol":1,"request_id":1,"ok":true,"data":{"sway":{"supported":true,"installed":true,"description":"Sway enabled","capabilities":{"windows":"supported","fullscreen":"supported","movement":"unsupported","pointer_control":"unsupported"}}}}
    );
    try std.testing.expect(model.sway_installed);
    try std.testing.expect(model.dirty());
    try std.testing.expect(std.mem.indexOf(u8, model.swayStatus(), "Movement: unsupported") != null);
    model.page = .platform;
    model.sway_prompt = true;
    const View = sdk.canvas.CompiledMarkupView(main.Model, main.Msg, main.app_markup);
    var ui = main.AppUi.init(arena.allocator());
    _ = try ui.finalize(View.build(&ui, model));
    try main.acceptResponse(model,
        \\{"protocol":1,"request_id":1,"ok":true,"data":{"sway":{"supported":true,"installed":false,"description":"Sway observations removed"}}}
    );
    try std.testing.expect(!model.sway_installed);
    try std.testing.expect(model.dirty());
    try std.testing.expectEqualStrings("true", model.fields[0].value());
}

test "Hyprland setup and removal retain the settings draft and separate capability limits" {
    var arena = std.heap.ArenaAllocator.init(std.testing.allocator);
    defer arena.deinit();
    const model = try arena.allocator().create(main.Model);
    model.* = .{};
    model.request_id = 1;
    try main.acceptResponse(model, @embedFile("fixtures/read.json"));
    model.fields[0].value_buffer.set("true");
    try main.acceptResponse(model,
        \\{"protocol":1,"request_id":1,"ok":true,"data":{"hyprland":{"supported":true,"installed":true,"description":"Hyprland enabled","capabilities":{"windows":"supported","fullscreen":"supported","movement":"unsupported","pointer_control":"unsupported"}}}}
    );
    try std.testing.expect(model.hyprland_installed);
    try std.testing.expect(model.dirty());
    try std.testing.expect(std.mem.indexOf(u8, model.hyprlandStatus(), "Movement: unsupported") != null);
    model.page = .platform;
    model.hyprland_prompt = true;
    const View = sdk.canvas.CompiledMarkupView(main.Model, main.Msg, main.app_markup);
    var ui = main.AppUi.init(arena.allocator());
    _ = try ui.finalize(View.build(&ui, model));
    try main.acceptResponse(model,
        \\{"protocol":1,"request_id":1,"ok":true,"data":{"hyprland":{"supported":true,"installed":false,"description":"Hyprland observations removed"}}}
    );
    try std.testing.expect(!model.hyprland_installed);
    try std.testing.expect(model.dirty());
    try std.testing.expectEqualStrings("true", model.fields[0].value());
}

test "GNOME actions use their own capability even when Hyprland differs" {
    var arena = std.heap.ArenaAllocator.init(std.testing.allocator);
    defer arena.deinit();
    const model = try arena.allocator().create(main.Model);
    model.* = .{ .gnome_supported = true, .hyprland_supported = false };
    const effects = try arena.allocator().create(main.Effects);
    effects.* = main.Effects.init(arena.allocator());
    defer effects.deinit();
    main.update(model, .gnome_setup, effects);
    try std.testing.expect(model.gnome_prompt);
    main.update(model, .gnome_cancel, effects);
    try std.testing.expect(!model.gnome_prompt);
    model.gnome_supported = false;
    model.hyprland_supported = true;
    main.update(model, .gnome_setup, effects);
    try std.testing.expect(!model.gnome_prompt);
    main.update(model, .gnome_confirm, effects);
    main.update(model, .gnome_remove, effects);
    try std.testing.expect(!model.busy);
    try std.testing.expectEqual(@as(u64, 0), model.request_id);
}

test "Gnome setup and removal retain the settings draft and separate capability limits" {
    var arena = std.heap.ArenaAllocator.init(std.testing.allocator);
    defer arena.deinit();
    const model = try arena.allocator().create(main.Model);
    model.* = .{};
    model.request_id = 1;
    try main.acceptResponse(model, @embedFile("fixtures/read.json"));
    model.fields[0].value_buffer.set("true");
    try main.acceptResponse(model,
        \\{"protocol":1,"request_id":1,"ok":true,"data":{"gnome":{"supported":true,"installed":true,"description":"Gnome enabled","capabilities":{"windows":"supported","fullscreen":"supported","movement":"unsupported","pointer_control":"unsupported"}}}}
    );
    try std.testing.expect(model.gnome_installed);
    try std.testing.expect(model.dirty());
    try std.testing.expect(std.mem.indexOf(u8, model.gnomeStatus(), "Movement: unsupported") != null);
    model.page = .platform;
    model.gnome_prompt = true;
    const View = sdk.canvas.CompiledMarkupView(main.Model, main.Msg, main.app_markup);
    var ui = main.AppUi.init(arena.allocator());
    _ = try ui.finalize(View.build(&ui, model));
    try main.acceptResponse(model,
        \\{"protocol":1,"request_id":1,"ok":true,"data":{"gnome":{"supported":true,"installed":false,"description":"Gnome observations removed"}}}
    );
    try std.testing.expect(!model.gnome_installed);
    try std.testing.expect(model.dirty());
    try std.testing.expectEqualStrings("true", model.fields[0].value());
}
