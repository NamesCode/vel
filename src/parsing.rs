// SPDX-FileCopyrightText: 2025 Name <lasagna@garfunkle.space>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{
    ast::{Dom, Element, ElementType},
    ComponentsCache, LazyDom,
};
use std::{
    borrow::BorrowMut,
    collections::{HashMap, HashSet},
    str::Chars,
    sync::{Arc, LazyLock},
};

/// A lazily evaluated static of all HTML5 void elements as of 2025-04-17
static VOID_ELEMENTS: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    HashSet::from([
        "!DOCTYPE", "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
        "param", "source", "track", "wbr",
    ])
});

struct BufferFrame {
    current_element: Element,
    page_index: usize,
    string_buffer: String,
}

impl BufferFrame {
    fn new(dom: Element) -> Self {
        BufferFrame {
            current_element: dom,
            page_index: 0,
            string_buffer: String::new(),
        }
    }
}

// TODO:
// - [X] Core parsing
// - [ ] Error handling *You aren't ever doing this you fucking moron*
// - [X] Fix bug where you can only parse one element
// - [ ] Variable passing (hx-params, slot)

fn parse_variable(char_iterator: &mut Chars, /*, inputs: &HashMap<String, String>*/) -> Element {
    Element::new(
        char_iterator.take_while(|char| char != &'}').collect(),
        HashMap::new(),
        ElementType::Variable,
    )
}

fn parse_attributes(char_iterator: &mut Chars) -> HashMap<String, Vec<Element>> {
    HashMap::from_iter(
        char_iterator
            .take_while(|char| char != &'>')
            .collect::<String>()
            .split_whitespace()
            .map(|sub_string| {
                if let Some((key, value)) = sub_string.split_once('=') {
                    let mut parsed_values = vec![];
                    let mut string_buffer = String::new();

                    while let Some(char) = value.chars().next() {
                        match char {
                            '{' => {
                                if !string_buffer.is_empty() {
                                    parsed_values.push(Element::new(
                                        String::new(),
                                        HashMap::new(),
                                        ElementType::Text(std::mem::take(&mut string_buffer)),
                                    ));
                                }

                                parsed_values.push(parse_variable(char_iterator))
                            }
                            char => string_buffer.push(char),
                        }
                    }

                    if !string_buffer.is_empty() {
                        parsed_values.push(Element::new(
                            String::new(),
                            HashMap::new(),
                            ElementType::Text(std::mem::take(&mut string_buffer)),
                        ));
                    }

                    (key.to_string(), parsed_values)
                } else {
                    (sub_string.to_string(), vec![])
                }
            }),
    )
}

fn parse_element(char_iterator: &mut Chars) -> Element {
    let mut open = true;

    let name: String = char_iterator
        .take_while(|char| {
            if char == &'>' {
                open = false;
                false
            } else {
                char != &' '
            }
        })
        .collect();

    if open {
        let attributes = parse_attributes(char_iterator);
    } else {
        let attributes = HashMap::new();
    };

    // Determine the kind
    match name.chars().next() {
        Some(char) if char.is_uppercase() => ElementType::Document,
        Some('/') => todo!(),
        Some(_) if VOID_ELEMENTS.contains(name.as_str()) => ElementType::Void,
        Some(_) => ElementType::Node,
        None => break 'char_loop,
    };

}

pub(crate) fn parse<'a>(component: &str, components: &mut ComponentsCache) -> Result<(), ()> {
    let mut buffer_stack = vec![BufferFrame::new(Element::new(
        component.to_string(),
        HashMap::new(),
        ElementType::Document,
    ))];
    let mut page_stack = vec![];

    let component_handle = components.get_mut(component).ok_or(())?;

    let page = if let LazyDom::Unparsed(page) = component_handle {
        std::mem::take(page)
    } else {
        #[cfg(debug_assertions)]
        eprintln!(
                "The tree has already been parsed. You should always check first so you can use the cached version"
            );

        return Ok(());
    };

    page_stack.push(page.chars());

    'buffer_loop: while let Some(mut frame) = buffer_stack.pop() {
        let page_index = frame.page_index;
        let char_iterator = &mut page_stack[page_index];

        'char_loop: while let Some(char) = char_iterator.next() {
            match char {
                '{' => frame
                    .current_element
                    .children
                    .push(Arc::new(parse_variable(char_iterator))),
                '<' => frame
                    .current_element
                    .children
                    .push(Arc::new(parse_element(char_iterator))),
                other_char => frame.string_buffer.push(other_char),
            }
        }

        // Since the dom and char_iterator are tied, we know that once the iterator loop is done,
        // so is the dom parsing.
        let _ = page_stack.remove(page_index);
        //*component_handle = LazyDom::Parsed(dom);
    }

    Ok(())
}
