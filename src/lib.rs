mod ast;
mod parsing;
mod rendering;

//#[cfg(test)]
//mod tests;

use ast::{Dom, Element};
use std::{borrow::Cow, collections::HashMap};

/// This'd make a bad partner :/
pub(crate) type LazyDom = ParseStatus<String, Dom>;

#[derive(Debug)]
pub(crate) enum ParseStatus<T, U> {
    Unparsed(T),
    Parsed(U),
}

#[derive(Debug)]
pub struct VelInstance {
    components: HashMap<String, LazyDom>,
}

impl VelInstance {
    pub fn new(components: HashMap<String, String>) -> Self {
        Self {
            components: HashMap::from_iter(
                components
                    .into_iter()
                    .map(|(key, value)| (key, LazyDom::Unparsed(value))),
            ),
        }
    }

    pub fn extend(&mut self, components: HashMap<String, String>) -> &mut Self {
        self.components.extend(
            components
                .into_iter()
                .map(|(key, value)| (key, LazyDom::Unparsed(value))),
        );
        self
    }

    pub fn render<F>(
        &self,
        component: String,
        inputs: HashMap<String, String>,
        rendering_callback: F,
        //) -> Result<String, RenderingError>>
    ) -> Result<String, ()>
    where
        F: Fn(Element) -> Option<Element> + std::marker::Copy,
    {
        let mut test: HashMap<(Vec<(&str, &str)>, &String, *const F), &str> = HashMap::new();

        let mut inputs_vec: Vec<_> = inputs
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect();
        inputs_vec.sort_unstable();
        dbg!(&inputs_vec);

        // NOTE: When making this fr, since we parse the whole thing into a DOM anyway, we can
        // figure out which inputs are required and filter based of that, reducing the amount of
        // redundant variations in the cache
        test.insert(
            (inputs_vec, &component, &rendering_callback as *const F),
            "hello world",
        );

        parsing::parse(&component, &self.components);
        //rendering::render(&dom, Cow::Borrowed(&inputs), &mut self.cache);
        Ok(String::new())
    }
}
