// -------------------------
//      <! IMPORTS !>
// -------------------------

const std = @import("std");
const Allocator = std.mem.Allocator;

pub const PointerEnum = union(enum) {
    Text: *Text,
    Variable: *Variable,
    Void: *Void,
    Node: *Node,
    Slot: *Slot,
    Link: *Link,
};

pub const ElementList = std.ArrayList(PointerEnum);

// Assuming the Elements are all on heap, *even if true*, leaves an icky taste in my mouth if we aren't explicitely initialising them as heap allocated.
pub fn deinitElementList(allocator: Allocator, element_list: *ElementList) void {
    for (element_list.items) |child| {
        switch (child) {
            .Text => |text_element| {
                text_element.deinit(allocator);
                allocator.destroy(text_element);
            },
            .Variable => |variable_element| {
                variable_element.deinit(allocator);
                allocator.destroy(variable_element);
            },
            .Void => |void_element| {
                void_element.deinit(allocator);
                allocator.destroy(void_element);
            },
            .Node => |node_element| {
                node_element.deinit(allocator);
                allocator.destroy(node_element);
            },
            .Slot => |slot_element| {
                slot_element.deinit(allocator);
                allocator.destroy(slot_element);
            },
            .Link => |link_element| {
                link_element.deinit(allocator);
                allocator.destroy(link_element);
            },
        }
    }
    element_list.deinit(allocator);
}

pub const AttributeValues = union(enum) {
    text: Text,
    variable: Variable,
};
pub const Attributes = std.StringHashMapUnmanaged(std.ArrayList(AttributeValues));

pub const Text = struct {
    value: []const u8,

    pub fn deinit(self: *@This(), allocator: Allocator) void {
        allocator.free(self.value);
    }
};

pub const Variable = struct {
    name: []const u8,

    pub fn deinit(self: *@This(), allocator: Allocator) void {
        allocator.free(self.name);
    }
};

pub const Void = struct {
    tag: []const u8,
    attributes: Attributes = Attributes.empty,

    pub fn deinit(self: *@This(), allocator: Allocator) void {
        allocator.free(self.tag);

        // const attribute_values = self.attributes.valueIterator();
        // allocator.free(attribute_values.items);
        self.attributes.deinit(allocator);
    }
};

pub const Node = struct {
    tag: []const u8,
    attributes: Attributes = Attributes.empty,
    children: ElementList = ElementList.empty,

    pub fn deinit(self: *@This(), allocator: Allocator) void {
        allocator.free(self.tag);

        // const attribute_values = self.attributes.valueIterator();
        // allocator.free(attribute_values.items);
        self.attributes.deinit(allocator);

        deinitElementList(allocator, &self.children);
    }
};

pub const Slot = struct {
    name: std.ArrayList(AttributeValues) = std.ArrayList(AttributeValues).empty,
    attributes: Attributes = Attributes.empty,
    children: ElementList = ElementList.empty,

    pub fn deinit(self: *@This(), allocator: Allocator) void {
        self.name.deinit(allocator);

        // const attribute_values = self.attributes.valueIterator();
        // allocator.free(attribute_values.items);
        self.attributes.deinit(allocator);

        deinitElementList(allocator, &self.children);
    }
};

pub const Link = struct {
    name: []const u8,
    attributes: Attributes,
    children: ElementList = ElementList.empty,

    pub fn deinit(self: *@This(), allocator: Allocator) void {
        allocator.free(self.name);

        // const attribute_values = self.attributes.valueIterator();
        // allocator.free(attribute_values.items);
        // self.attributes.deinit(allocator);

        // const slot_content_values = self.slot_content.valueIterator();
        // allocator.free(slot_content_values.items);
        // self.slot_content.deinit(allocator);
    }
};

/// Checks if a element tag is a HTML5 void elements. Updated 2025-04-17
pub fn tagIsVoid(tag: []const u8) bool {
    for ([_][]const u8{
        "!DOCTYPE",
        "area",
        "base",
        "br",
        "col",
        "embed",
        "hr",
        "img",
        "input",
        "link",
        "meta",
        "param",
        "source",
        "track",
        "wbr",
    }) |void_tag| {
        if (std.mem.eql(u8, void_tag, tag)) return true;
    }

    return false;
}

const testing = std.testing;

test "isVoidElement works" {
    const test_element = Void{ .tag = "img", .attributes = undefined };
    try testing.expect(tagIsVoid(test_element.tag));
}
