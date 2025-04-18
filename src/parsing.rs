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

enum PushTarget {
    Children,
    Attributes((String, Vec<Arc<Element>>)),
}

struct BufferFrame {
    current_element: Element,
    push_to: PushTarget,
    page_index: usize,
    string_buffer: String,
}

impl BufferFrame {
    fn new(dom: Element) -> Self {
        BufferFrame {
            current_element: dom,
            push_to: PushTarget::Children,
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
        ElementType::Variable,
    )
}

fn parse_attribute(char_iterator: &mut Chars) -> Vec<Arc<Element>> {
    let mut in_quotes = false;
    let mut buffer = String::new();
    let mut collected_elements = Vec::new();

    // parses the first part of the element
    while let Some(char) = char_iterator.next() {
        match char {
            '"' | '\'' => in_quotes = !in_quotes,
            ' ' if !in_quotes => break,
            '<' if !in_quotes => todo!(),
            '>' if !in_quotes => {
                if !buffer.is_empty() {
                    //collected_parts.push(buffer.clone());
                    buffer.clear();
                }
                break;
            }
            '{' => collected_elements.push(Arc::new(parse_variable(char_iterator))),
            other_char => buffer.push(other_char),
        }
    }

    collected_elements
}

pub(crate) fn parse<'a>(component: &str, components: &mut ComponentsCache) -> Result<(), ()> {
    let mut buffer_stack = vec![BufferFrame::new(Element::new(
        component.to_string(),
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
                '{' => match frame.push_to {
                    PushTarget::Children => frame
                        .current_element
                        .children
                        .push(Arc::new(parse_variable(char_iterator))),
                    PushTarget::Attributes((name, elements)) => todo!(),
                },
                '<' => {
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

                    let push_to = if open {
                        PushTarget::Attributes((String::new(), vec![]))
                    } else {
                        PushTarget::Children
                    };

                    // Determine the kind
                    match name.chars().next() {
                        Some(char) if char.is_uppercase() => {
                            buffer_stack.push(frame);

                            match components.get(&name) {
                                Some(LazyDom::Unparsed(page)) => {
                                    page_stack.push(page.chars());

                                    buffer_stack.push(BufferFrame {
                                        current_element: Element::new(name, ElementType::Document),
                                        push_to,
                                        page_index: page_stack.len(),
                                        string_buffer: String::new(),
                                    });
                                }
                                Some(LazyDom::Parsed(dom)) => {
                                    buffer_stack.push(BufferFrame {
                                        current_element: (*dom.tree).clone(),
                                        push_to,
                                        page_index,
                                        string_buffer: String::new(),
                                    });
                                }
                                None => todo!(),
                            }
                        }
                        Some(char) if char == '/' => {
                            // BUG: This will not work if this is the first frame...
                            if let Some(previous_frame) = buffer_stack.last_mut() {
                                match &previous_frame.push_to {
                                    PushTarget::Children => previous_frame
                                        .current_element
                                        .children
                                        .push(Arc::new(frame.current_element)),
                                    PushTarget::Attributes((name, elements)) => todo!(),
                                }
                            };
                        }
                        Some(_) if VOID_ELEMENTS.contains(name.as_str()) => match push_to {
                            PushTarget::Children => {
                                if let Some(previous_frame) = buffer_stack.last_mut() {
                                    match &previous_frame.push_to {
                                        PushTarget::Children => previous_frame
                                            .current_element
                                            .children
                                            .push(Arc::new(Element::new(name, ElementType::Void))),
                                        PushTarget::Attributes((name, elements)) => todo!(),
                                    }
                                };
                            }
                            PushTarget::Attributes((_, _)) => {
                                buffer_stack.push(frame);
                                buffer_stack.push(BufferFrame {
                                    current_element: Element::new(name, ElementType::Void),
                                    push_to,
                                    page_index,
                                    string_buffer: String::new(),
                                })
                            }
                        },
                        Some(_) => {
                            ElementType::Node;
                        }
                        None => break 'char_loop,
                    };

                    continue 'buffer_loop;
                }
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
