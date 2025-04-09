use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
pub struct Dom {
    tree: Arc<Element>,
}

impl Dom {
    pub fn new(component_name: String) -> Self {
        Dom {
            tree: Arc::new(Element::new(
                component_name,
                HashMap::new(),
                ElementType::Document,
            )),
        }
    }
}

#[derive(Debug)]
pub enum ElementType {
    /// Page doctype
    Doctype(String),
    /// The page document. This is always the first node.
    Document,
    /// Node type elements
    Node,
    // HACK: I really don't like this interface and I think it's going to make a rough api to use.
    // Atleast I can hide it with interfaces
    /// Usize should always be derived with `Arc::as_ptr(<First "Document" ElementType>) as usize`
    Slot(RwLock<Vec<(usize, Arc<Element>)>>),
    /// Just text. Concats to Variable
    Text(String),
    /// If a string comes before or after this during rendering, they will be attached to eachother
    Variable(String),
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
        ElementType::Slot(RwLock::new(vec![(
            Arc::as_ptr(&dom.tree) as usize,
            element,
        )]))
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    /// The elements children
    pub children: Vec<Arc<Element>>,
    /// The element name; e.g. <p> = "p"
    pub name: String,
    /// The elements attributes; e.g. <p style='color: red'> = ("style", "color: red")
    pub attributes: HashMap<String, String>,
    /// The elements contained value, if it is None then that just means it has not been parsed
    /// yet.
    pub kind: ElementType,
}

impl Element {
    pub fn new(name: String, attributes: HashMap<String, String>, kind: ElementType) -> Self {
        Element {
            children: vec![],
            name,
            attributes,
            kind,
        }
    }
}
