const std = @import("std");
const main = @import("main.zig");
const sdk = @import("native_sdk");

test "settings wheel motion settles locally, reverses, clamps and honors reduced motion" {
    const tokens = main.designTokens(&.{});
    inline for (.{ @as(f32, 1000.0 / 30.0), @as(f32, 1000.0 / 60.0), @as(f32, 1000.0 / 120.0) }) |dt| {
        var state = sdk.canvas.ScrollState{ .viewport_extent = 400, .content_extent = 4000 };
        state = state.applyWheelClamped(40, tokens.scroll);
        const first = state.offset;
        try std.testing.expect(first > 0 and first < 20);
        var elapsed: f32 = 0;
        while (state.needsKineticStep(tokens.scroll) and elapsed < 500) : (elapsed += dt) {
            const next = state.stepKinetic(dt, tokens.scroll);
            try std.testing.expect(next.offset >= state.offset);
            try std.testing.expect(next.offset - state.offset < 30);
            state = next;
        }
        try std.testing.expect(!state.needsKineticStep(tokens.scroll));
        try std.testing.expect(elapsed < 350);
        try std.testing.expect(state.offset > 45 and state.offset < 85);
        const settled = state.offset;
        state = state.stepKinetic(1000, tokens.scroll);
        try std.testing.expectEqual(settled, state.offset);

        state = state.applyWheelClamped(120, tokens.scroll);
        state = state.stepKinetic(dt, tokens.scroll);
        const before_reverse = state.offset;
        state = state.applyWheelClamped(-40, tokens.scroll).stepKinetic(dt, tokens.scroll);
        try std.testing.expect(state.offset < before_reverse);
        try std.testing.expect(state.velocity < 0);
        state = state.applyWheelClamped(-10000, tokens.scroll);
        try std.testing.expectEqual(@as(f32, 0), state.offset);
        try std.testing.expectEqual(@as(f32, 0), state.velocity);

        var fractional = sdk.canvas.ScrollState{ .viewport_extent = 400, .content_extent = 4000 };
        fractional = fractional.applyWheelClamped(10, tokens.scroll);
        elapsed = 0;
        while (fractional.needsKineticStep(tokens.scroll) and elapsed < 500) : (elapsed += dt) {
            fractional = fractional.stepKinetic(dt, tokens.scroll);
        }
        try std.testing.expect(fractional.offset > 5 and fractional.offset < 25);
        try std.testing.expect(elapsed < 350);
        try std.testing.expect(!fractional.needsKineticStep(tokens.scroll));

        var repeated = sdk.canvas.ScrollState{ .offset = 300, .viewport_extent = 400, .content_extent = 4000 };
        for (0..6) |_| {
            const previous = repeated.offset;
            repeated = repeated.applyWheelClamped(40, tokens.scroll).stepKinetic(dt, tokens.scroll);
            try std.testing.expect(repeated.offset > previous);
        }
        const before_immediate_reverse = repeated.offset;
        repeated = repeated.applyWheelClamped(-10, tokens.scroll);
        try std.testing.expect(repeated.offset < before_immediate_reverse);
        try std.testing.expect(repeated.velocity < 0);
        state = state.applyWheelClamped(100000, tokens.scroll);
        try std.testing.expectEqual(state.maxOffset(), state.offset);
        try std.testing.expectEqual(@as(f32, 0), state.velocity);
    }
    const reduced = main.designTokens(&.{ .system_appearance = .{ .reduce_motion = true } });
    const still = (sdk.canvas.ScrollState{ .viewport_extent = 400, .content_extent = 4000 }).applyWheelClamped(40, reduced.scroll);
    try std.testing.expectEqual(@as(f32, 40), still.offset);
    try std.testing.expect(!still.needsKineticStep(reduced.scroll));
    try std.testing.expectEqual(still.offset, still.stepKinetic(500, reduced.scroll).offset);
}

test "native frame delays advance wheel decay by elapsed time without changing nominal cadence" {
    const harness = try sdk.TestHarness().create(std.testing.allocator, .{});
    defer harness.destroy(std.testing.allocator);
    harness.null_platform.gpu_surfaces = true;
    var context: u8 = 0;
    const app = sdk.App{ .context = &context, .name = "scroll clock regression", .source = sdk.platform.WebViewSource.html("<h1>Settings</h1>") };
    try harness.start(app);
    _ = try harness.runtime.createView(.{ .window_id = 1, .label = "canvas", .kind = .gpu_surface, .frame = sdk.geometry.RectF.init(0, 0, 320, 240) });
    const items = [_]sdk.canvas.Widget{.{ .id = 3, .kind = .button, .text = "Bottom", .frame = sdk.geometry.RectF.init(0, 3900, 250, 32) }};
    const children = [_]sdk.canvas.Widget{.{ .id = 2, .kind = .scroll_view, .frame = sdk.geometry.RectF.init(12, 12, 280, 200), .children = &items }};
    var nodes: [8]sdk.canvas.WidgetLayoutNode = undefined;
    const layout = try sdk.canvas.layoutWidgetTree(.{ .id = 1, .kind = .panel, .children = &children }, sdk.geometry.RectF.init(0, 0, 320, 240), &nodes);
    _ = try harness.runtime.setCanvasWidgetLayout(1, "canvas", layout);
    const tokens = main.designTokens(&.{});
    _ = try harness.runtime.emitCanvasWidgetDisplayList(1, "canvas", tokens);
    const nominal = 16_666_667;
    var stamp: u64 = 1_000_000_000;
    try harness.runtime.dispatchPlatformEvent(app, .{ .gpu_surface_frame = .{ .label = "canvas", .size = sdk.geometry.SizeF.init(320, 240), .frame_index = 1, .timestamp_ns = stamp, .frame_interval_ns = nominal } });
    try harness.runtime.dispatchPlatformEvent(app, .{ .gpu_surface_input = .{ .window_id = 1, .label = "canvas", .kind = .scroll, .x = 50, .y = 70, .delta_y = 40, .timestamp_ns = stamp + 1 } });
    const immediate = harness.runtime.views[0].widget_scroll_states[1];
    try std.testing.expect(immediate.offset > 0 and immediate.velocity > 0);
    // The input's responding frame is skipped, so a prior idle gap cannot become a jump.
    stamp += 5_000_000_000;
    try harness.runtime.dispatchPlatformEvent(app, .{ .gpu_surface_frame = .{ .label = "canvas", .size = sdk.geometry.SizeF.init(320, 240), .frame_index = 2, .timestamp_ns = stamp, .frame_interval_ns = nominal } });
    try std.testing.expectEqual(immediate.offset, harness.runtime.views[0].widget_scroll_states[1].offset);
    var expected = immediate;
    var previous_offset = immediate.offset;
    for ([_]u64{ 31, 17, 48, 24, 33, 40, 20, 47, 33 }, 0..) |elapsed_ms, frame| {
        stamp += elapsed_ms * 1_000_000;
        // Synchronously presented hover input must not reset wheel decay.
        try harness.runtime.dispatchPlatformEvent(app, .{ .gpu_surface_input = .{ .window_id = 1, .label = "canvas", .kind = .pointer_move, .x = 50, .y = 70, .timestamp_ns = stamp - 1_000_000 } });
        harness.runtime.views[0].recordGpuSurfaceInputLatencyForFrame(stamp - 500_000);
        expected = expected.stepKinetic(@floatFromInt(elapsed_ms), tokens.scroll);
        try harness.runtime.dispatchPlatformEvent(app, .{ .gpu_surface_frame = .{ .label = "canvas", .size = sdk.geometry.SizeF.init(320, 240), .frame_index = frame + 3, .timestamp_ns = stamp, .frame_interval_ns = nominal } });
        const actual = harness.runtime.views[0].widget_scroll_states[1];
        try std.testing.expect(actual.offset >= previous_offset and actual.offset < 85);
        try std.testing.expectApproxEqAbs(expected.velocity, actual.velocity, 0.01);
        try std.testing.expectEqual(@as(u64, nominal), harness.runtime.views[0].gpu_frame_interval_ns);
        previous_offset = actual.offset;
    }
    try std.testing.expect(!expected.needsKineticStep(tokens.scroll));
    try std.testing.expect(previous_offset > 45 and previous_offset < 85);
    // A new notch after idle rebases, and a later long pause consumes bounded
    // decay without turning elapsed time into an enormous Euler displacement.
    stamp += 5_000_000_000;
    try harness.runtime.dispatchPlatformEvent(app, .{ .gpu_surface_input = .{ .window_id = 1, .label = "canvas", .kind = .scroll, .x = 50, .y = 70, .delta_y = 40, .timestamp_ns = stamp } });
    const after_idle_input = harness.runtime.views[0].widget_scroll_states[1];
    harness.runtime.views[0].recordGpuSurfaceInputLatencyForFrame(stamp + 1);
    stamp += nominal;
    try harness.runtime.dispatchPlatformEvent(app, .{ .gpu_surface_frame = .{ .label = "canvas", .size = sdk.geometry.SizeF.init(320, 240), .frame_index = 20, .timestamp_ns = stamp, .frame_interval_ns = nominal } });
    const after_idle_frame = harness.runtime.views[0].widget_scroll_states[1];
    const expected_first_frame = after_idle_input.stepKinetic(1000.0 / 60.0, tokens.scroll);
    try std.testing.expect(after_idle_frame.velocity > 0);
    try std.testing.expectApproxEqAbs(expected_first_frame.offset, after_idle_frame.offset, 0.01);
    try std.testing.expectApproxEqAbs(expected_first_frame.velocity, after_idle_frame.velocity, 0.01);
    stamp += 5_000_000_000;
    try harness.runtime.dispatchPlatformEvent(app, .{ .gpu_surface_frame = .{ .label = "canvas", .size = sdk.geometry.SizeF.init(320, 240), .frame_index = 21, .timestamp_ns = stamp, .frame_interval_ns = nominal } });
    const paused = harness.runtime.views[0].widget_scroll_states[1];
    try std.testing.expectEqual(@as(f32, 0), paused.velocity);
    try std.testing.expect(paused.offset - previous_offset > 45 and paused.offset - previous_offset < 85);
}

test "older service Wayland values remain stored but only Linux displays the control" {
    var arena = std.heap.ArenaAllocator.init(std.testing.allocator);
    defer arena.deinit();
    var model = main.Model{};
    model.request_id = 1;
    try main.acceptResponse(&model, @embedFile("fixtures/read.json"));
    model.page = .platform;
    var stored: usize = 0;
    for (model.fields[0..model.field_count]) |f| {
        if (std.mem.eql(u8, f.key(), "platform.wayland")) stored += 1;
    }
    var visible: usize = 0;
    for (model.visible(arena.allocator())) |f| {
        if (std.mem.eql(u8, f.key(), "platform.wayland")) visible += 1;
    }
    try std.testing.expectEqual(@as(usize, 1), stored);
    try std.testing.expectEqual(@as(usize, if (@import("builtin").os.tag == .linux) 1 else 0), visible);
}

test "wheel-only scroll changes publish accessibility geometry without idle churn" {
    const Platform = struct {
        updates: usize = 0,
        anchor_y: f32 = -999,
        fn value(self: *@This()) sdk.platform.Platform {
            return .{
                .context = self,
                .name = "scroll geometry regression",
                .surface_value = .{ .id = 1, .size = sdk.geometry.SizeF.init(320, 240), .scale_factor = 1 },
                .run_fn = run,
                .services = .{ .context = self, .load_webview_fn = load, .create_view_fn = create, .update_widget_accessibility_fn = publish },
            };
        }
        fn run(_: *anyopaque, _: sdk.platform.EventHandler, _: *anyopaque) !void {}
        fn load(_: ?*anyopaque, _: sdk.platform.WebViewSource) !void {}
        fn create(_: ?*anyopaque, _: sdk.platform.ViewOptions) !void {}
        fn publish(context: ?*anyopaque, snapshot: sdk.platform.WidgetAccessibilitySnapshot) !void {
            const self: *@This() = @ptrCast(@alignCast(context.?));
            self.updates += 1;
            for (snapshot.nodes) |node| {
                if (node.id == 3) self.anchor_y = node.bounds.y;
            }
        }
    };
    var platform = Platform{};
    const runtime = try std.testing.allocator.create(sdk.Runtime);
    defer std.testing.allocator.destroy(runtime);
    sdk.Runtime.initAt(runtime, .{ .platform = platform.value() });
    const app = sdk.App{ .context = &platform, .name = "wheel geometry", .source = sdk.platform.WebViewSource.html("<h1>Settings</h1>") };
    try runtime.dispatchPlatformEvent(app, .app_start);
    _ = try runtime.createView(.{ .window_id = 1, .label = "canvas", .kind = .gpu_surface, .frame = sdk.geometry.RectF.init(0, 0, 320, 240) });
    const items = [_]sdk.canvas.Widget{
        .{ .id = 3, .kind = .button, .text = "Anchor", .frame = sdk.geometry.RectF.init(0, 100, 250, 32) },
        .{ .id = 4, .kind = .button, .text = "Bottom", .frame = sdk.geometry.RectF.init(0, 468, 250, 32) },
    };
    const children = [_]sdk.canvas.Widget{.{ .id = 2, .kind = .scroll_view, .value = 40, .frame = sdk.geometry.RectF.init(12, 12, 280, 200), .children = &items }};
    var nodes: [8]sdk.canvas.WidgetLayoutNode = undefined;
    const layout = try sdk.canvas.layoutWidgetTree(.{ .id = 1, .kind = .panel, .children = &children }, sdk.geometry.RectF.init(0, 0, 320, 240), &nodes);
    _ = try runtime.setCanvasWidgetLayout(1, "canvas", layout);
    _ = try runtime.emitCanvasWidgetDisplayList(1, "canvas", main.designTokens(&.{}));
    const initial_y = platform.anchor_y;
    const initial_updates = platform.updates;
    try runtime.dispatchPlatformEvent(app, .{ .gpu_surface_input = .{ .window_id = 1, .label = "canvas", .kind = .scroll, .x = 50, .y = 70, .delta_y = -10000 } });
    // Publication stays off the input path and is flushed after the responding frame.
    try std.testing.expectEqual(initial_updates, platform.updates);
    try runtime.dispatchPlatformEvent(app, .{ .gpu_surface_frame = .{ .label = "canvas", .size = sdk.geometry.SizeF.init(320, 240), .frame_index = 1 } });
    var scrolled = try runtime.canvasWidgetLayout(1, "canvas");
    try std.testing.expectEqual(@as(f32, 0), scrolled.findById(2).?.widget.value);
    try std.testing.expectEqual(@as(f32, 0), runtime.views[0].widget_scroll_states[0].velocity);
    try std.testing.expectApproxEqAbs(initial_y + 40, platform.anchor_y, 0.01);

    _ = try runtime.emitCanvasWidgetDisplayList(1, "canvas", main.designTokens(&.{ .system_appearance = .{ .reduce_motion = true } }));
    const reset_y = platform.anchor_y;
    try runtime.dispatchPlatformEvent(app, .{ .gpu_surface_input = .{ .window_id = 1, .label = "canvas", .kind = .scroll, .x = 50, .y = 70, .delta_y = 40 } });
    try runtime.dispatchPlatformEvent(app, .{ .gpu_surface_frame = .{ .label = "canvas", .size = sdk.geometry.SizeF.init(320, 240), .frame_index = 2 } });
    scrolled = try runtime.canvasWidgetLayout(1, "canvas");
    try std.testing.expectEqual(@as(f32, 40), scrolled.findById(2).?.widget.value);
    try std.testing.expectEqual(@as(f32, 0), runtime.views[0].widget_scroll_states[0].velocity);
    try std.testing.expectApproxEqAbs(reset_y - 40, platform.anchor_y, 0.01);
    const settled_updates = platform.updates;
    for (0..5) |frame| {
        _ = try runtime.stepCanvasWidgetKineticScroll(1, "canvas", 16);
        try runtime.dispatchPlatformEvent(app, .{ .gpu_surface_frame = .{ .label = "canvas", .size = sdk.geometry.SizeF.init(320, 240), .frame_index = 3 + frame } });
    }
    try std.testing.expectEqual(settled_updates, platform.updates);
}

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
