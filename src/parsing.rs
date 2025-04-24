// SPDX-FileCopyrightText: 2025 Name <lasagna@garfunkle.space>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{
    ast::{elements::*, Dom},
    ComponentsCache, LazyDom,
};
use frames::PageFrame;
use std::{
    collections::HashMap,
    ops::Deref,
    pin::Pin,
    str::Chars,
    sync::{Arc, Mutex},
};

/// A lazily evaluated static of all HTML5 void elements as of 2025-04-17
const VOID_ELEMENTS: [&str; 15] = [
    "!DOCTYPE", "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
    "param", "source", "track", "wbr",
];

/// We modularize the page logic so that we can safely handle all edge cases where the type *could*
/// error in theory but wont as it's an unreachable!() state
mod frames {
    use super::{Arc, Chars, Document, Element, Mpreggable, Mutex};

    #[derive(PartialEq, Eq)]
    enum PushTarget {
        // TODO: Attributes,
        Children,
        Slot,
    }

    pub type PageStack<'a> = Vec<PageFrame<'a>>;

    pub struct PageFrame<'a> {
        mpregee: Mpreggable,
        pub page: Arc<Mutex<Chars<'a>>>,
        push_target: PushTarget,
        pub string_buffer: String,
    }

    impl<'a> PageFrame<'a> {
        pub fn new_document(name: String, page: Chars<'a>) -> Self {
            PageFrame {
                mpregee: Mpreggable::Document(Document::new(name)),
                page: Arc::new(Mutex::new(page)),
                push_target: PushTarget::Children,
                string_buffer: String::new(),
            }
        }

        pub fn new_slot(document: Document, page: Arc<Mutex<Chars<'a>>>) -> Self {
            PageFrame {
                mpregee: Mpreggable::Document(document),
                page,
                push_target: PushTarget::Slot,
                string_buffer: String::new(),
            }
        }

        pub fn new_child(element: Mpreggable, page: Arc<Mutex<Chars<'a>>>) -> Self {
            PageFrame {
                mpregee: element,
                page,
                push_target: PushTarget::Children,
                string_buffer: String::new(),
            }
        }

        pub fn push_element(&mut self, element: Element) {
            match self.push_target {
                PushTarget::Slot => {
                    if self.mpregee.try_add_slot(element).is_err() {
                        unreachable!("This should not happen as the initialiser functions protect against this case. Please check their logic TwT.")
                    }
                }
                PushTarget::Children => self.mpregee.add_child(element),
            }
        }

        pub fn try_take_my_kids(&mut self, document: Document) -> Result<(), ()> {
            if self.push_target == PushTarget::Slot {
                if let Mpreggable::Document(slot_document) = &mut self.mpregee {
                    slot_document.children = document.children.clone();
                    return Ok(());
                }
            }
            Err(())
        }

        pub fn try_close(
            self,
            name: String,
            parent_frame: Option<&mut PageFrame<'a>>,
        ) -> Result<Option<Document>, ()> {
            if self.mpregee.get_name() == name {
                match self.mpregee {
                    Mpreggable::Document(document) => match self.push_target {
                        PushTarget::Children => return Ok(Some(document)),
                        PushTarget::Slot => parent_frame
                            .ok_or(())?
                            .push_element(Element::Document(document)),
                    },
                    Mpreggable::Slot(_) | Mpreggable::Node(_) => {
                        parent_frame.ok_or(())?.push_element(self.mpregee.into())
                    }
                }
                Ok(None)
            } else {
                // Names dont match
                Err(())
            }
        }
    }
}

/// Used to determine the current element in the frame stack.
/// Only elements that can gain kids can be the current element.
enum Mpreggable {
    Document(Document),
    Node(Node),
    Slot(Slot),
}

impl Mpreggable {
    #[inline]
    fn add_child(&mut self, child_element: Element) {
        match self {
            Self::Document(document) => document.children.push(Arc::new(child_element)),
            Self::Node(node) => node.children.push(Arc::new(child_element)),
            Self::Slot(slot) => slot.children.push(Arc::new(child_element)),
        }
    }

    #[inline]
    fn get_name(&self) -> &str {
        match self {
            Self::Document(document) => &document.name,
            Self::Node(node) => &node.name,
            Self::Slot(_) => "Slot",
        }
    }

    fn try_add_slot(&mut self, mut element: Element) -> Result<(), ()> {
        if let Mpreggable::Document(document) = self {
            let name = match &mut element {
                Element::Document(document) => {
                    document
                        .attributes
                        .remove("slot")
                        .unwrap_or(vec![AttributeValues::Text(Text {
                            value: "default".to_string(),
                        })])
                }

                Element::Node(node) => {
                    node.attributes
                        .remove("slot")
                        .unwrap_or(vec![AttributeValues::Text(Text {
                            value: "default".to_string(),
                        })])
                }

                Element::Slot(slot) => {
                    slot.attributes
                        .remove("slot")
                        .unwrap_or(vec![AttributeValues::Text(Text {
                            value: "default".to_string(),
                        })])
                }

                Element::Text(_) | Element::Variable(_) => vec![AttributeValues::Text(Text {
                    value: "default".to_string(),
                })],

                Element::Void(void) => {
                    void.attributes
                        .remove("slot")
                        .unwrap_or(vec![AttributeValues::Text(Text {
                            value: "default".to_string(),
                        })])
                }
            };

            let new_element = Arc::new(element);

            document
                .slot_content
                .entry(name)
                .and_modify(|value| value.push(new_element.clone()))
                .or_insert(vec![new_element]);

            Ok(())
        } else {
            Err(())
        }
    }
}

impl From<Mpreggable> for Element {
    #[inline]
    fn from(mpreggable_element: Mpreggable) -> Self {
        match mpreggable_element {
            Mpreggable::Document(document) => Self::Document(document),
            Mpreggable::Node(node) => Self::Node(node),
            Mpreggable::Slot(slot) => Self::Slot(slot),
        }
    }
}

// TODO:
// - [X] Core parsing
// - [ ] Error handling *You aren't ever doing this you fucking moron*
// - [X] Fix bug where you can only parse one element
// - [ ] Variable passing (hx-params, slot)

fn parse_variable(char_iterator: &mut Chars, /*, inputs: &HashMap<String, String>*/) -> Variable {
    Variable {
        name: char_iterator.take_while(|char| char != &'}').collect(),
    }
}

// WARN: Wtf is this shit ass code
fn parse_attributes(char_iterator: &mut Chars) -> Attributes {
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
                                    parsed_values.push(AttributeValues::Text(Text {
                                        value: std::mem::take(&mut string_buffer),
                                    }));
                                }

                                parsed_values
                                    .push(AttributeValues::Variable(parse_variable(char_iterator)))
                            }
                            char => string_buffer.push(char),
                        }
                    }

                    if !string_buffer.is_empty() {
                        parsed_values.push(AttributeValues::Text(Text {
                            value: std::mem::take(&mut string_buffer),
                        }));
                    }

                    (key.to_string(), parsed_values)
                } else {
                    (sub_string.to_string(), vec![])
                }
            }),
    )
}

enum ParsingAction {
    ParseInnards(Mpreggable),
    Mpreg(Element),
    Close(String),
    ExhaustedChars,
}

fn parse_element(char_iterator: &mut Chars) -> ParsingAction {
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

    let mut attributes = Attributes::new();

    if open {
        attributes = parse_attributes(char_iterator);
    };

    // Determine the kind
    match name.chars().next() {
        Some(_) if name.as_str() == "Slot" => {
            let name = attributes
                .remove("name")
                .unwrap_or(vec![AttributeValues::Text(Text {
                    value: "default".to_string(),
                })]);

            ParsingAction::ParseInnards(Mpreggable::Slot(Slot {
                name,
                attributes,
                children: vec![],
            }))
        }
        Some(char) if char.is_uppercase() => {
            ParsingAction::ParseInnards(Mpreggable::Document(Document {
                name,
                attributes,
                slot_content: HashMap::new(),
                children: vec![],
            }))
        }
        Some('/') => ParsingAction::Close(name[1..].to_string()),
        Some(_) if VOID_ELEMENTS.contains(&name.as_str()) => {
            ParsingAction::Mpreg(Element::Void(Void { name, attributes }))
        }
        Some(_) => ParsingAction::ParseInnards(Mpreggable::Node(Node {
            name,
            attributes,
            children: vec![],
        })),
        None => ParsingAction::ExhaustedChars,
    }
}

pub(crate) fn parse(component: &str, components: &mut ComponentsCache) -> Result<(), ()> {
    let mut page_stack = vec![];
    let component_handle = components.remove(component).ok_or(())?;

    let page = if let LazyDom::Unparsed(page) = component_handle {
        page
    } else {
        #[cfg(debug_assertions)]
        eprintln!(
                "The tree has already been parsed. You should always check first so you can use the cached version"
            );

        return Ok(());
    };

    page_stack.push(page);
    let mut frame_stack = vec![PageFrame::new_document(
        component.to_string(),
        page_stack[0].chars(),
    )];

    'frame_loop: while let Some(mut frame) = frame_stack.pop() {
        let page = frame.page.clone();
        let mut char_iterator = page.lock().expect("char_iterator mutex poisoned. I don't know what we could do here so it's best to panic as it's unrecoverable at the moment.");

        'char_loop: while let Some(char) = char_iterator.next() {
            match char {
                '{' => frame.push_element(Element::Variable(parse_variable(&mut char_iterator))),
                '<' => match parse_element(&mut char_iterator) {
                    ParsingAction::Mpreg(element) => frame.push_element(element),
                    ParsingAction::ParseInnards(element) => {
                        let page = frame.page.clone();

                        // Unlocks the char_iterator Mutex
                        drop(char_iterator);

                        frame_stack.push(frame);

                        if let Mpreggable::Document(document) = element {
                            match components.remove(&document.name) {
                                Some(LazyDom::Parsed(dom)) => {
                                    frame_stack
                                        .push(PageFrame::new_slot(dom.tree.deref().clone(), page));
                                }
                                Some(LazyDom::Unparsed(dom_page)) => {
                                    page_stack.push(dom_page);
                                    let name = document.name.clone();
                                    frame_stack.push(PageFrame::new_slot(document, page));
                                    frame_stack.push(PageFrame::new_document(
                                        name,
                                        page_stack[page_stack.len()].chars(),
                                    ));
                                }
                                None => return Err(()),
                            }
                        } else {
                            frame_stack.push(PageFrame::new_child(element, page));
                        }

                        continue 'frame_loop;
                    }
                    ParsingAction::Close(name) => {
                        let document = frame.try_close(name, frame_stack.last_mut())?;

                        drop(char_iterator);

                        if let Some(document) = document {
                            match frame_stack.last_mut() {
                                Some(frame) => {
                                    frame.try_take_my_kids(document.clone())?;
                                    components.insert(
                                        document.name.clone(),
                                        LazyDom::Parsed(Dom::new(document)),
                                    );
                                }
                                None => {
                                    components.insert(
                                        document.name.clone(),
                                        LazyDom::Parsed(Dom::new(document)),
                                    );
                                    return Ok(());
                                }
                            }
                        }

                        continue 'frame_loop;
                    }
                    ParsingAction::ExhaustedChars => break 'char_loop,
                },
                '\\' => {
                    if let Some(char) = char_iterator.next() {
                        frame.string_buffer.push(char)
                    } else {
                        break 'char_loop;
                    }
                }
                other_char => frame.string_buffer.push(other_char),
            }
        }
    }

    Ok(())
}
