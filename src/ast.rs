// SPDX-FileCopyrightText: 2025 Name <lasagna@garfunkle.space>
//
// SPDX-License-Identifier: EUPL-1.2

use elements::Document;
use std::{fmt::Debug, sync::Arc};

#[derive(Debug, Clone)]
pub struct Dom {
    pub(crate) tree: Arc<Document>,
}

impl Dom {
    pub fn new(root: Document) -> Self {
        Dom {
            tree: Arc::new(root),
        }
    }
}

pub mod elements {
    use super::Arc;
    use std::{collections::HashMap, fmt::Debug};

    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    pub enum AttributeValues {
        Text(Text),
        Variable(Variable),
    }

    #[derive(Debug, Clone)]
    pub enum Element {
        Document(Document),
        Node(Node),
        Slot(Slot),
        Text(Text),
        /// If a string comes before or after this during rendering, they will be attached to eachother
        Variable(Variable),
        Void(Void),
    }

    /// Just a shorthand alias of HashMap<String, AttributeValues> for use within the crate
    pub(crate) type Attributes = HashMap<String, Vec<AttributeValues>>;

    /// The document element for the page.
    /// Because this is always the top level element, we can clone it for storing Dom-depending state data here for
    /// other elements like Slot or attribute inputs!
    #[derive(Debug, Clone)]
    pub struct Document {
        pub name: String,
        pub attributes: Attributes,
        pub slot_content: HashMap<Vec<AttributeValues>, Vec<Arc<Element>>>,
        pub children: Vec<Arc<Element>>,
    }

    impl Document {
        pub fn new(name: String) -> Self {
            Self {
                name,
                attributes: Attributes::new(),
                slot_content: HashMap::new(),
                children: vec![],
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Node {
        pub name: String,
        pub attributes: Attributes,
        pub children: Vec<Arc<Element>>,
    }

    #[derive(Debug, Clone)]
    pub struct Slot {
        pub name: Vec<AttributeValues>,
        pub attributes: Attributes,
        pub children: Vec<Arc<Element>>,
    }

    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    pub struct Text {
        pub value: String,
    }

    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    pub struct Variable {
        pub name: String,
    }

    #[derive(Debug, Clone)]
    pub struct Void {
        pub name: String,
        pub attributes: Attributes,
    }
}
