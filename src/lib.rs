// SPDX-FileCopyrightText: 2025 Name <lasagna@garfunkle.space>
//
// SPDX-License-Identifier: EUPL-1.2

mod ast;
mod parsing;
mod rendering;

//#[cfg(test)]
//mod tests;

use ast::{elements::Element, Dom};
use std::collections::HashMap;

/// This'd make a bad partner :/
pub(crate) type LazyDom = ParseStatus<String, Dom>;
pub(crate) type ComponentsCache = HashMap<String, LazyDom>;

#[derive(Debug, Clone)]
pub(crate) enum ParseStatus<T, U> {
    Unparsed(T),
    Parsed(U),
}

#[derive(Debug)]
pub struct VelInstance {
    components: ComponentsCache,
}

impl VelInstance {
    pub fn new(components: HashMap<String, String>) -> Self {
        Self {
            components: ComponentsCache::from_iter(
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
        &mut self,
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

        parsing::parse(&component, &mut self.components)?;
        //rendering::render(&dom, Cow::Borrowed(&inputs), &mut self.cache);
        Ok(String::new())
    }

    pub fn parse(&mut self, component: String) -> Result<(), ()> {
        parsing::parse(&component, &mut self.components)?;
        Ok(())
    }
}

#[test]
fn i_want_to_kms() {}
