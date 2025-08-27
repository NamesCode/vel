const std = @import("std");
const io = std.io;

const Allocator = std.mem.Allocator;
const ArenaAllocator = std.heap.ArenaAllocator;

const elements = @import("elements.zig");

// TODO: I would like to have nicer contextual error messages for this, like maybe providing an `ErrorDetail` struct that can be formatted into a nice trace.
// Potentially take an argument to the entry function of an anonymous config struct or a `std.ArrayList(ErrorDetail)`.
const ParsingError = error{ CloseTagButNoOpenElement, UnclosedElements, MalformedElement, EmptyTag };
const Error = ParsingError || Allocator.Error;

const PositionalData = struct {
    line: usize = 0,
    character: usize = 0,

    fn addLine(self: *@This()) void {
        self.character = 0;
        self.line += 1;
    }
};

const PositionalReader = struct {
    position: PositionalData,
    reader: io.Reader,

    const Self = @This();

    fn init(reader: io.Reader) Self {
        return .{ .position = PositionalData{}, .reader = reader };
    }

    fn takeByte(self: *Self) io.Reader.Error!u8 {
        const byte = try self.reader.takeByte();

        if (byte == '\n') {
            self.position.addLine();
        } else {
            self.position.character += 1;
        }

        return byte;
    }
};

/// Stores the parsing state for our parser.
/// Initialise with `init` and deinitialise with `deinit` to drop everything or `close_state`
const ParserState = struct {
    // HACK: THIS IS INEFFICIENT AF. **F I X   I T**.
    // BUG: This design is flawed. Not every interface is a `*elements.ElementList`.
    // For example, if the current element is a Link, we know it can only hold `*Slot` but are now forced to make it accept the ENTIRE `PointerEnum` types.
    // This is a **bad api**.
    element_stack: elements.ElementList,
    top_level_elements: elements.ElementList,
    text_buffer: std.ArrayList(u8),

    const Self = @This();

    fn init(allocator: Allocator) !Self {
        return .{
            .element_stack = try std.ArrayList(elements.PointerEnum).initCapacity(allocator, 4),
            .top_level_elements = try std.ArrayList(elements.PointerEnum).initCapacity(allocator, 4),
            .text_buffer = try std.ArrayList(u8).initCapacity(allocator, 100),
        };
    }

    fn deinit(self: *Self, allocator: Allocator) void {
        elements.deinitElementList(allocator, &self.element_stack);
        elements.deinitElementList(allocator, &self.top_level_elements);
        self.text_buffer.deinit(allocator);
    }

    /// Helper function for figuring out where to push child elements too.
    fn mpreg_here(self: *Self) *elements.ElementList {
        if (self.element_stack.getLastOrNull()) |mpregee| {
            switch (mpregee) {
                .Text, .Variable, .Void => unreachable,
                .Node => |node| return &node.children,
                .Slot => |slot| return &slot.children,
                .Link => |link| return &link.children,
            }
        } else return &self.top_level_elements;
    }

    fn mpreg(self: *Self, allocator: Allocator, child: elements.PointerEnum) !void {
        switch (child) {
            .Text, .Variable, .Void => try self.mpreg_here().append(allocator, child),
            .Node, .Slot, .Link => try self.element_stack.append(allocator, child),
        }
    }

    fn close_element(self: *Self, allocator: Allocator) !void {
        // BUG: This will close the element regardless of whether it is the correct closing tag.
        const element = self.element_stack.pop() orelse return ParsingError.CloseTagButNoOpenElement;
        try self.mpreg_here().append(allocator, element);
    }

    fn drain_text_buffer(self: *Self, allocator: Allocator) !?elements.Text {
        if (self.text_buffer.items.len == 0) return null;

        self.text_buffer.shrinkAndFree(allocator, self.text_buffer.items.len);

        const text_slice = try self.text_buffer.toOwnedSlice(allocator);
        self.text_buffer = try std.ArrayList(u8).initCapacity(allocator, 100);

        return elements.Text{ .value = text_slice };
    }

    /// This will deinit() the ParserState for you, there is no need to do so after this.
    /// All elements should be closed or else this will throw a parsing error.
    fn close_state(self: *Self, allocator: Allocator) !elements.ElementList {
        if (self.element_stack.items.len != 0) return ParsingError.UnclosedElements;
        self.element_stack.deinit(allocator);

        if (self.text_buffer.items.len != 0) {
            // We convert the `text_buffer` to a `Text` node for the last time.
            self.text_buffer.shrinkAndFree(allocator, self.text_buffer.items.len);

            const final_text_element_ptr = try allocator.create(elements.Text);
            final_text_element_ptr.* = elements.Text{ .value = try self.text_buffer.toOwnedSlice(allocator) };

            try self.top_level_elements.append(allocator, elements.PointerEnum{ .Text = final_text_element_ptr });
        } else {
            self.text_buffer.deinit(allocator);
        }

        // Minimises the memory usage of the `ArrayList` by resizing its capacity to the number of actual items.
        self.top_level_elements.shrinkAndFree(allocator, self.top_level_elements.items.len);
        return self.top_level_elements;
    }
};

fn parse_variable(comptime T: anytype, allocator: Allocator, reader: *T) !elements.Variable {
    if (!@hasDecl(T, "takeByte")) {
        @compileError(@typeName(T) ++ " doesn't have the field `takeByte()`. This must be implemented for the reader.\n");
    }

    var name_buffer = try std.ArrayList(u8).initCapacity(allocator, 15);

    while (reader.takeByte() catch null) |current_character| {
        if (current_character == '}') {
            break;
        }

        try name_buffer.append(allocator, current_character);
    }

    name_buffer.shrinkAndFree(allocator, name_buffer.items.len);
    const name_slice = try name_buffer.toOwnedSlice(allocator);

    return elements.Variable{ .name = name_slice };
}

fn parse_attributes(comptime T: anytype, allocator: Allocator, reader: *T) !elements.Attributes {
    if (!@hasDecl(T, "takeByte")) {
        @compileError(@typeName(T) ++ " doesn't have the field `takeByte()`. This must be implemented for the reader.\n");
    }

    // TODO: Parse attributes

    // var attributes = elements.Attributes.empty;
    //
    // return attributes;

    _ = allocator;
    _ = reader;

    return elements.Attributes.empty; // HACK: Doing this for now, just so it compiles.
}

const ElementInformation = union(enum) {
    ClosingTag,
    Void: elements.Void,
    Node: elements.Node,
    Slot: elements.Slot,
    Link: elements.Link,
};

fn parse_element(comptime T: anytype, allocator: Allocator, reader: *T) !ElementInformation {
    if (!@hasDecl(T, "takeByte")) {
        @compileError(@typeName(T) ++ " doesn't have the field `takeByte()`. This must be implemented for the reader.\n");
    }

    // I reserve 8 bytes here because the longest html tag name is 8 bytes.
    var string_buffer = try std.ArrayList(u8).initCapacity(allocator, 8);
    errdefer string_buffer.deinit(allocator);

    var closed = false;

    while (reader.takeByte() catch null) |current_character| {
        switch (current_character) {
            ' ' => break,
            '>' => {
                closed = true;
                break;
            },
            '\\' => try string_buffer.append(allocator, reader.takeByte() catch {
                return ParsingError.MalformedElement;
            }),
            else => try string_buffer.append(allocator, current_character),
        }
    }

    const first_char_ptr = if (string_buffer.items.len == 0) return ParsingError.EmptyTag else string_buffer.items[0];

    if (first_char_ptr == '/') {
        string_buffer.deinit(allocator);
        return ElementInformation.ClosingTag;
    }

    string_buffer.shrinkAndFree(allocator, string_buffer.items.len);

    var attributes = if (!closed) try parse_attributes(T, allocator, reader) else elements.Attributes.empty;

    const tag = try string_buffer.toOwnedSlice(allocator);

    if (std.ascii.isUpper(first_char_ptr)) {
        return ElementInformation{
            .Link = elements.Link{ .name = tag, .attributes = attributes },
        };
    } else if (elements.tagIsVoid(tag)) {
        return ElementInformation{
            .Void = elements.Void{ .tag = tag, .attributes = attributes },
        };
    } else if (std.mem.eql(u8, tag, "slot")) {
        var name: std.ArrayList(elements.AttributeValues) = undefined;
        if (attributes.getEntry("name")) |entry| {
            allocator.free(entry.key_ptr.*);
            name = entry.value_ptr.*;
        } else name = std.ArrayList(elements.AttributeValues).empty;

        return ElementInformation{
            .Slot = elements.Slot{ .name = name, .attributes = attributes },
        };
    } else {
        return ElementInformation{
            .Node = elements.Node{ .tag = tag, .attributes = attributes },
        };
    }
}

pub fn parse_component(reader: io.Reader, allocator: Allocator) Error!elements.ElementList {
    var positional_reader = PositionalReader.init(reader);

    var parser_state = try ParserState.init(allocator);
    errdefer parser_state.deinit(allocator);

    // The only error this will return is `error.EndOfStream` so we can safely ignore errors.
    while (positional_reader.takeByte() catch null) |current_character| {
        switch (current_character) {
            '{' => {
                if (try parser_state.drain_text_buffer(allocator)) |text_element| {
                    const text_element_pointer = try allocator.create(elements.Text);
                    errdefer allocator.destroy(text_element_pointer);

                    text_element_pointer.* = text_element;
                    errdefer text_element_pointer.deinit(allocator);

                    try parser_state.mpreg(allocator, elements.PointerEnum{ .Text = text_element_pointer });
                }

                const variable_pointer = try allocator.create(elements.Variable);
                variable_pointer.* = try parse_variable(PositionalReader, allocator, &positional_reader);
                try parser_state.mpreg(allocator, elements.PointerEnum{ .Variable = variable_pointer });
            },
            '<' => {
                switch (try parse_element(PositionalReader, allocator, &positional_reader)) {
                    .ClosingTag => try parser_state.close_element(allocator),
                    .Void => |void_element| {
                        const void_pointer = try allocator.create(elements.Void);
                        void_pointer.* = void_element;
                        try parser_state.mpreg(allocator, elements.PointerEnum{ .Void = void_pointer });
                    },
                    .Node => |node| {
                        const node_pointer = try allocator.create(elements.Node);
                        node_pointer.* = node;
                        try parser_state.mpreg(allocator, elements.PointerEnum{ .Node = node_pointer });
                    },
                    .Slot => |slot| {
                        // BUG: causes memory leak. Fix.
                        const slot_pointer = try allocator.create(elements.Slot);
                        slot_pointer.* = slot;
                        try parser_state.mpreg(allocator, elements.PointerEnum{ .Slot = slot_pointer });
                    },
                    .Link => |link| {
                        const link_pointer = try allocator.create(elements.Link);
                        link_pointer.* = link;
                        try parser_state.mpreg(allocator, elements.PointerEnum{ .Link = link_pointer });
                    },
                }
            }, // TODO: Handle parsing an element and the hell that entails
            '\\' => try parser_state.text_buffer.append(allocator, positional_reader.takeByte() catch {
                break;
            }),
            else => try parser_state.text_buffer.append(allocator, current_character),
        }
    }

    return parser_state.close_state(allocator);
}

// -------------------------
//      <! TESTING !>
// -------------------------

const testing = std.testing;

test "preview parse output" {
    const gpa = testing.allocator;

    const template = io.Reader.fixed("Hello World! {foo} \\{bar} <balls></balls><img><errs-slot></slot><Document></Document>");

    var tl_element_array = try parse_component(template, gpa);
    defer elements.deinitElementList(gpa, &tl_element_array);

    std.debug.print("Parsing output:\n", .{});
    for (tl_element_array.items) |element_pointer| {
        std.debug.print("{s}\n", .{@tagName(element_pointer)});
    }
}
