// -------------------------
//      <! IMPORTS !>
// -------------------------

const std = @import("std");
const io = std.io;
const Allocator = std.mem.Allocator;

const elements = @import("elements.zig");
const parsing = @import("parsing.zig");

// --------------------------
//   <! TYPE DEFINITIONS !>
// --------------------------

/// The same allocator must be used throughout its entire lifetime.
const Engine = struct {
    component_registry: std.StringHashMap(elements.PointerEnum),

    const Self = @This();

    fn init(allocator: Allocator) Self {
        return .{
            .component_registry = std.StringHashMap(elements.PointerEnum).init(allocator),
        };
    }

    fn deinit(self: *Self) void {
        self.component_registry.deinit();
    }

    /// This will eagerly parse the component. The `name` keys memory is managed by the caller, only free when the engine is freed or when this component is freed using `freeComponent`.
    // NOTE: In future I may make this an OwnedSlice since I am unhappy with this model of managing the keys memory.
    // The engine knows exactly when the name should be freed and as such it should be the one to free it.
    fn addComponent(self: *Self, name: []const u8, template: io.AnyReader) !void {
        const component_tree = try parsing.parse_component(template, Self.allocator);

        try self.component_registry.put(name, component_tree);
    }
};

// -------------------------
//      <! TESTING !>
// -------------------------

// const testing = std.testing;

// test "create components map" {
//     const allocator = std.testing.allocator;
//
//     var engine = Engine.init(allocator);
//     defer engine.deinit();
//
//     try engine.addComponent("ligma", "balls");
// }
