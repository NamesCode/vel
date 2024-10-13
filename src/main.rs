use std::collections::HashMap;

mod vel {
    use core::fmt;
    use std::{
        collections::{HashMap, HashSet},
        fmt::Display,
    };

    #[derive(Debug)]
    pub struct ArenaTree<T> {
        arena: Vec<Node<T>>,
        dead_nodes: Vec<usize>,
    }

    impl<T> ArenaTree<T> {
        fn new() -> Self {
            ArenaTree {
                arena: vec![],
                dead_nodes: vec![],
            }
        }

        fn add_node(&mut self, parent: Option<usize>, data: T) -> usize {
            if let Some(node_index) = self.dead_nodes.pop() {
                let node = &mut self.arena[node_index];
                node.parent = parent;
                node.data = Some(data);
                node_index
            } else {
                let index = self.arena.len();
                if let Some(parent_index) = parent {
                    self.arena[parent_index].add_child(index);
                }
                self.arena.push(Node::new(index, parent, data));
                index
            }
        }

        fn remove_node(&mut self, index: usize) {
            self.dead_nodes.push(index);
            let node = &mut self.arena[index];
            node.data = None;

            let mut stack: Vec<usize> = node.children.drain(..).collect();
            if let Some(parent_index) = node.parent {
                self.arena[parent_index].delete_child(index)
            }

            while let Some(index) = stack.pop() {
                self.dead_nodes.push(index);
                let node = &mut self.arena[index];
                node.data = None;
                stack.extend(node.children.drain(..));
            }
        }
    }

    /// If data is None then that means that this Node is dead and can be safely removed or
    /// repurposed.
    #[derive(Debug)]
    struct Node<T> {
        index: usize,
        parent: Option<usize>,
        children: Vec<usize>,
        data: Option<T>,
    }

    impl<T> Node<T> {
        fn new(index: usize, parent: Option<usize>, data: T) -> Self {
            Node {
                index,
                parent,
                children: vec![],
                data: Some(data),
            }
        }

        fn add_child(&mut self, child: usize) {
            self.children.push(child);
        }

        fn delete_child(&mut self, child: usize) {
            self.children.retain(|&stored_child| stored_child != child)
        }
    }

    /// The Tag of the element is always the first element in the attributes vector.
    ///
    /// If the Tag contains a empty string then this means it is text element.
    #[derive(Debug)]
    pub struct DOMElement<'a> {
        void_element: bool,
        attributes: Vec<ElementAttributes<'a>>,
        text: &'a str,
    }

    impl<'a> DOMElement<'a> {
        /// The Tag field MUST be ElementAttributes::Tag()
        fn new(
            tag: ElementAttributes<'a>,
            mut attributes: Vec<ElementAttributes<'a>>,
            text: &'a str,
            void_element: bool,
        ) -> Self {
            if let ElementAttributes::Tag(_) = tag {
                attributes.insert(0, tag);
                DOMElement {
                    void_element,
                    attributes,
                    text,
                }
            } else {
                panic!("Tag field can only be ElementAttributes::Tag()!")
            }
        }
    }

    #[derive(Debug)]
    pub enum ElementAttributes<'a> {
        /// Represents an HTML tag (e.g. "div", "p")
        Tag(&'a str),
        /// Represents an element's ID (e.g. "header")
        Id(&'a str),
        /// Represents an element's class (e.g. "navbar")
        Class(&'a str),
        /// Represents any other attribute (name, value)
        Other(&'a str, &'a str),
    }

    /// Test
    #[derive(Debug)]
    pub enum EditableField {
        /// Specifies the nodes text itself is editable
        Text,
        /// Specifies the nodes which node attribute is editable
        Attribute(usize),
    }

    pub type DOM<'a> = ArenaTree<DOMElement<'a>>;

    #[derive(Debug)]
    pub struct VelDOM<'a> {
        /// This contains an array of all the nodes which can be substitued for new text
        editable_nodes: Vec<(usize, EditableField)>,
        /// The underlying DOM
        dom: DOM<'a>,
    }

    impl<'a> VelDOM<'a> {
        pub fn render(ignore: HashSet<Vec<ElementAttributes>>) {
            todo!()
        }
    }

    #[derive(Debug)]
    pub struct VelInstance<'a> {
        components: HashMap<&'a str, &'a str>,
    }

    impl Display for VelInstance<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "hamburger")
        }
    }

    impl<'a> VelInstance<'a> {
        pub fn new(components: HashMap<&'a str, &'a str>) -> Self {
            Self { components }
        }

        pub fn extend(&mut self, components: HashMap<&'a str, &'a str>) -> &mut Self {
            self.components.extend(components);
            self
        }

        pub fn parse(
            &'a self,
            component: &'a str,
            inputs: HashMap<&str, &str>,
            //) -> Result<&Self, Err> {
        ) -> VelDOM {
            println!("{}", component);
            dbg!(&self.components);
            let _page = self
                .components
                .get(component)
                .expect("could get the component numbskull");
            let mut dom = VelDOM::new();

            dom
        }
    }
}

use crate::vel::{ArenaTree, DOMElement, VelInstance};

fn main() {
    let test = VelInstance::new(HashMap::from([("Component", "banana")]));
    dbg!(&test);
    let test_dom = test.parse("Component", HashMap::from([("data", "cheese")]));
    dbg!(&test_dom);
}

// * Read file -> Tree -> Render
// Take arguements -> Parse file -> Return html
