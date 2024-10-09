use std::collections::HashMap;

mod vel {
    use core::fmt;
    use std::{collections::HashMap, fmt::Display};

    /*
    #[derive(Debug)]
    pub struct ParserError {
        kind: ParserErrorKind,
    }

    #[derive(Debug)]
    pub enum ParserErrorKind {
        ComponentNotFound,
    }

    impl fmt::Display for ParserErrorKind {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::ComponentNotFound => write!(f, "beee Component"),
            }
        }
    }

    impl std::error::Error for ParserErrorKind {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                _ => None,
            }
        }
    }

    type Err = ParserErrorKind;
    */
    #[derive(Debug)]
    pub struct VelInstance<'a> {
        components: &'a HashMap<&'a str, &'a str>,
    }

    impl Display for VelInstance<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "hamburger")
        }
    }

    impl<'a> VelInstance<'a> {
        pub fn new(components: &'a HashMap<&'a str, &'a str>) -> Self {
            Self { components }
        }

        /*
        pub fn extend(&mut self, components: HashMap<&'a str, &'a str>) -> &mut Self {
            self.components.extend(components);
            self
        }
        */

        pub fn parse(
            &'a self,
            component: &'a str,
            inputs: HashMap<&str, &str>,
            //) -> Result<&Self, Err> {
        ) -> &Self {
            println!("{}", component);
            dbg!(&self.components);
            let _page = self
                .components
                .get(component)
                .expect("could get the component numbskull");
            self
        }

        pub fn strip(&mut self) -> &mut Self {
            // TODO: Take CSS selector enum to decide what elements to strip
            todo!()
        }
    }
}

use crate::vel::VelInstance;

fn main() {
    println!("Hello, world!");
    let pages: HashMap<&str, &str> = HashMap::from([("Component", "banana")]);

    let test = VelInstance::new(&PAGES);
    dbg!(&test);
    test.parse("Component", HashMap::from([("data", "cheese")]));
}

// * Read file -> Tree -> Render
// Take arguements -> Parse file -> Return html
