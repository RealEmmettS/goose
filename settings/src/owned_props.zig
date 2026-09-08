//! The private Linux prop protocol carries data, never paths or native handles.
const std = @import("std");

const max_input = 4 * 1024 * 1024;
const max_pixels = 900 * 700 * 4;
const Operation = enum(u32) { note, image, move, passthrough, focus, text, close, shutdown };
const Wire = struct {
    v: u8,
    op: Operation,
    id: u64 = 0,
    x: i32 = 0,
    y: i32 = 0,
    width: u32 = 0,
    height: u32 = 0,
    pixel_width: u32 = 0,
    pixel_height: u32 = 0,
    passthrough: bool = false,
    title: []const u8 = "",
    text: []const u8 = "",
    pixels: []const u8 = "",
};

const Command = extern struct {
    id: u64,
    operation: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    pixel_width: u32,
    pixel_height: u32,
    passthrough: i32,
    title: [*:0]const u8,
    text: [*:0]const u8,
    pixels: [*]const u8,
    pixels_len: usize,
};

extern fn honk_props_run(light: [*]const u8, light_len: usize, bold: [*]const u8, bold_len: usize) c_int;
extern fn honk_props_apply(command: *const Command) c_int;

pub fn run() !void {
    const light = @embedFile("fonts/Makira-Light.ttf");
    const bold = @embedFile("fonts/Makira-Bold.ttf");
    if (honk_props_run(light.ptr, light.len, bold.ptr, bold.len) != 0) return error.OwnedPropsUnavailable;
}

fn validText(text: []const u8, limit: usize) bool {
    return text.len <= limit and std.mem.indexOfScalar(u8, text, 0) == null and std.unicode.utf8ValidateSlice(text);
}

fn parse(allocator: std.mem.Allocator, input: []const u8) !Command {
    if (input.len == 0 or input.len > max_input) return error.InvalidCommand;
    const parsed = try std.json.parseFromSlice(Wire, allocator, input, .{
        .ignore_unknown_fields = false,
        .allocate = .alloc_always,
        .max_value_len = max_input,
    });
    defer parsed.deinit();
    const wire = parsed.value;
    if (wire.v != 1 or (wire.op != .shutdown and wire.id == 0) or
        wire.x < -1_000_000 or wire.x > 1_000_000 or wire.y < -1_000_000 or wire.y > 1_000_000 or
        !validText(wire.title, 256) or !validText(wire.text, 16384)) return error.InvalidCommand;
    if (wire.op == .note or wire.op == .image) {
        if (wire.width == 0 or wire.width > 900 or wire.height == 0 or wire.height > 700) return error.InvalidCommand;
    }
    var pixels: []u8 = &.{};
    if (wire.op == .image) {
        if (wire.pixel_width == 0 or wire.pixel_width > 900 or wire.pixel_height == 0 or wire.pixel_height > 700) return error.InvalidCommand;
        const size = try std.base64.standard.Decoder.calcSizeForSlice(wire.pixels);
        if (size > max_pixels or size != @as(usize, wire.pixel_width) * wire.pixel_height * 4) return error.InvalidCommand;
        pixels = try allocator.alloc(u8, size);
        try std.base64.standard.Decoder.decode(pixels, wire.pixels);
        var index: usize = 0;
        while (index < size) : (index += 4) {
            const alpha = pixels[index + 3];
            if (pixels[index] > alpha or pixels[index + 1] > alpha or pixels[index + 2] > alpha) return error.InvalidCommand;
        }
    } else if (wire.pixels.len != 0 or wire.pixel_width != 0 or wire.pixel_height != 0) return error.InvalidCommand;
    return .{
        .id = wire.id,
        .operation = @intFromEnum(wire.op),
        .x = wire.x,
        .y = wire.y,
        .width = wire.width,
        .height = wire.height,
        .pixel_width = wire.pixel_width,
        .pixel_height = wire.pixel_height,
        .passthrough = @intFromBool(wire.passthrough),
        .title = try allocator.dupeZ(u8, wire.title),
        .text = try allocator.dupeZ(u8, wire.text),
        .pixels = pixels.ptr,
        .pixels_len = pixels.len,
    };
}

pub fn decode(input: [*]const u8, length: usize) callconv(.c) c_int {
    if (length == 0 or length > max_input) return 0;
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const command = parse(arena.allocator(), input[0..length]) catch return 0;
    return honk_props_apply(&command);
}

test "private command preserves Unicode and rejects malformed or unbounded inputs" {
    var arena = std.heap.ArenaAllocator.init(std.testing.allocator);
    defer arena.deinit();
    const allocator = arena.allocator();
    const note = try parse(allocator,
        \\{"v":1,"op":"note","id":42,"width":400,"height":250,"title":"A note"}
    );
    try std.testing.expectEqual(@as(u64, 42), note.id);
    const text = try parse(allocator,
        \\{"v":1,"op":"text","id":42,"text":"Café 🦆\nhello"}
    );
    try std.testing.expectEqualStrings("Café 🦆\nhello", std.mem.span(text.text));
    for ([_][]const u8{
        \\{"v":2,"op":"note","id":1,"width":400,"height":250}
        ,
        \\{"v":1,"op":"note","id":0,"width":400,"height":250}
        ,
        \\{"v":1,"op":"note","id":1,"width":4000,"height":250}
        ,
        \\{"v":1,"op":"text","id":1,"text":"bad\u0000text"}
        ,
        \\{"v":1,"op":"move","id":1,"x":1000001}
        ,
        \\{"v":1,"op":"focus","id":1,"window_id":123}
        ,
        \\{"v":1,"op":"execute","id":1}
        ,
        \\{"v":1,"op":"image","id":1,"width":400,"height":250,"pixel_width":1,"pixel_height":1,"pixels":"AAAA"}
        ,
    }) |invalid| {
        if (parse(allocator, invalid)) |_| return error.InvalidCommandAccepted else |_| {}
    }
    const image = try parse(allocator,
        \\{"v":1,"op":"image","id":1,"width":100,"height":100,"pixel_width":1,"pixel_height":1,"pixels":"/wAA/w=="}
    );
    try std.testing.expectEqualSlices(u8, &.{ 255, 0, 0, 255 }, image.pixels[0..image.pixels_len]);
    try std.testing.expectEqual(@as(usize, 72), @sizeOf(Command));
}
