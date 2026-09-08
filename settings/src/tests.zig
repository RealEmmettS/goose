const std = @import("std");
const main = @import("main.zig");
const sdk = @import("native_sdk");

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

test "KDE permission responses and native consent dialog preserve unsaved settings" {
    var arena = std.heap.ArenaAllocator.init(std.testing.allocator);
    defer arena.deinit();
    const model = try arena.allocator().create(main.Model);
    model.* = .{};
    model.request_id = 1;
    try main.acceptResponse(model, @embedFile("fixtures/read.json"));
    model.fields[0].value_buffer.set("true");
    try main.acceptResponse(model,
        \\{"protocol":1,"request_id":1,"ok":true,"data":{"integrations":{"supported":true,"installed":true,"description":"KDE companion enabled","capabilities":{"windows":"supported","movement":"supported","pointer_observation":"supported","pointer_control":"unsupported","fullscreen":"supported","dnd":"unsupported","prop_positioning":"unsupported"}}}}
    );
    try std.testing.expect(model.dirty());
    try std.testing.expect(model.kde_installed);
    try std.testing.expect(std.mem.indexOf(u8, model.integrationStatus(), "Control: unsupported") != null);
    model.kde_prompt = true;
    const View = sdk.canvas.CompiledMarkupView(main.Model, main.Msg, main.app_markup);
    var ui = main.AppUi.init(arena.allocator());
    _ = try ui.finalize(View.build(&ui, model));
    try main.acceptResponse(model,
        \\{"protocol":1,"request_id":1,"ok":true,"data":{"integrations":{"supported":true,"installed":false,"description":"KDE integration is off"}}}
    );
    try std.testing.expect(!model.kde_installed);
    try std.testing.expect(model.dirty());
    try std.testing.expectEqualStrings("true", model.fields[0].value());
}
