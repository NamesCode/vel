// SPDX-FileCopyrightText: 2025 Name <lasagna@garfunkle.space>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
pub struct Dom {
    pub(crate) tree: Arc<Element>,
}

impl Dom {
    pub fn new(root: Element) -> Self {
        Dom {
            tree: Arc::new(root),
        }
    }
}

#[derive(Debug)]
pub enum ElementType {
    /// Page doctype
    Doctype(String),
    /// Root tag for a document
    Document,
    /// Node type elements
    Node,
    /// The string is always the component name of the component that slotted it
    Slot(RwLock<Vec<(String, Arc<Element>)>>),
    /// Just text. Concats to Variable
    Text(String),
    /// If a string comes before or after this during rendering, they will be attached to eachother
    Variable,
    /// Void type elements
    Void,
}

impl Clone for ElementType {
    fn clone(&self) -> Self {
        match self {
            ElementType::Slot(lock) => match lock.read() {
                Ok(value) => ElementType::Slot(RwLock::new(value.clone())),
                Err(error) => {
                    eprintln!("The lock was poisoned when attempting to clone the RwLock of a Slot element. This is an issue of Vel, and probably not you, the user, so please report this. Here is the full error: {}", error);
                    ElementType::Slot(RwLock::new(vec![]))
                }
            },
            other => other.clone(),
        }
    }
}

impl ElementType {
    /// Helper function for building a slot
    pub fn slot_builder(dom: Dom, element: Arc<Element>) -> ElementType {
        ElementType::Slot(RwLock::new(vec![(dom.name, element)]))
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    /// The elements children
    pub children: Vec<Arc<Element>>,
    /// The element name; e.g. <p> = "p"
    pub name: String,
    /// The elements attributes; e.g. <p style='color: red'> = ("style", "color: red")
    pub attributes: HashMap<String, Vec<Arc<Element>>>,
    /// The elements contained value, if it is None then that just means it has not been parsed
    /// yet.
    pub kind: ElementType,
}

impl Element {
    pub fn new(
        name: String,
        kind: ElementType,
    ) -> Self {
        Element {
            children: vec![],
            name,
            attributes: HashMap::new(),
            kind,
        }
    }
}
