const std = @import("std");
const hello = @import("hello.zig");

pub fn main() !void {
    const a = 2;
    const b = 2;
    const added = hello.add(a, b);
    std.log.info("This is some stuff added: {d} + {d} = {d}", .{ a, b, added });
}

test "simple test" {
    var list = std.ArrayList(i32).init(std.testing.allocator);
    defer list.deinit(); // try commenting this out and see if zig detects the memory leak!
    try list.append(42);
    try std.testing.expectEqual(@as(i32, 42), list.pop());
}
