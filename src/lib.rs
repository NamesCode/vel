use core::fmt;
use std::{collections::HashMap, fmt::Display, str::Chars};

#[derive(Debug)]
pub struct VelInstance<'a> {
    components: HashMap<&'a str, &'a str>,
    cache: HashMap<String, String>,
}

impl Display for VelInstance<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "hamburger")
    }
}

impl<'a> VelInstance<'a> {
    pub fn new(components: HashMap<&'a str, &'a str>) -> Self {
        Self {
            components,
            cache: HashMap::new(),
        }
    }

    pub fn extend(&mut self, components: HashMap<&'a str, &'a str>) -> &mut Self {
        self.components.extend(components);
        self
    }

    pub fn render(
        &'_ mut self,
        component: String,
        inputs: HashMap<String, String>,
        //) -> Result<&Self, Err> {
    ) -> String {
        fn parse_variable(char_iterator: &mut Chars, inputs: &HashMap<String, String>) -> String {
            let variable_name: String = char_iterator.take_while(|char| char != &'}').collect();
            inputs
                .get(variable_name.as_str())
                .map(|value| value.to_string())
                .unwrap_or_else(|| format!("{{{}}}", variable_name))
        }

        fn parse_tags(
            parent: &mut VelInstance<'_>,
            char_iterator: &mut Chars,
            inputs: &HashMap<String, String>,
        ) -> String {
            if let Some(first_char) = char_iterator.next() {
                if first_char.is_uppercase() {
                    // State vars
                    let mut in_quotes = false;

                    let mut buffer = String::from(first_char);
                    // The end of each element is specified by an empty character in the array.
                    // CAN BE CHANGED IN FUTURE
                    let mut attributes: Vec<String> = vec![];

                    while let Some(char) = char_iterator.next() {
                        /*
                        dbg!(&char);
                        dbg!(&buffer);
                        dbg!(&attributes);
                        */

                        // NOTE: Whilst this is more complex than it needs to be, it gives us
                        // the opportunity to do things like Svelte's slot in future.
                        match char {
                            '"' | '\'' => in_quotes = !in_quotes,
                            ' ' | '>' => {
                                if !in_quotes {
                                    attributes.push(buffer.clone());
                                    buffer.clear()
                                }
                            }
                            '{' => buffer.push_str(&parse_variable(char_iterator, inputs)),
                            '<' => {
                                if !in_quotes {
                                    attributes.push(buffer.clone());
                                    buffer.clear();

                                    let tag_value = parse_tags(parent, char_iterator, inputs);
                                    if tag_value.contains(format!("/{}", first_char).as_str()) {
                                        break;
                                    } else {
                                        attributes.push(tag_value);
                                        buffer.clear();
                                    }
                                }
                            }
                            other_char => buffer.push(other_char),
                        }
                    }

                    let mut inputs = inputs.clone();
                    inputs.extend(
                        attributes
                            .iter()
                            .skip(1)
                            .take_while(|attribute| !attribute.is_empty())
                            .map(|attribute| {
                                let (key, value) =
                                    attribute.split_once('=').unwrap_or((attribute, attribute));
                                (key.to_owned(), value.to_owned())
                            }),
                    );

                    parent.render(attributes[0].to_owned(), inputs)
                } else {
                    format!(
                        "<{}{}>",
                        first_char,
                        char_iterator
                            .take_while(|char| char != &'>')
                            .collect::<String>()
                    )
                }
            } else {
                "<".to_string()
            }
        }

        return if let Some(cached_page) = self.cache.get(component.as_str()) {
            cached_page.to_string()
        } else if let Some(page) = self.components.get(component.as_str()) {
            let mut return_page = String::with_capacity(page.len());
            let mut char_iterator = page.chars();
            let mut cachable = true;

            while let Some(char) = char_iterator.next() {
                match char {
                    '{' => {
                        cachable = false;
                        let variable_data = parse_variable(&mut char_iterator, &inputs);
                        return_page.push_str(&variable_data);
                    }
                    '<' => {
                        cachable = false;
                        let element_data = parse_tags(self, &mut char_iterator, &inputs);
                        return_page.push_str(&element_data);
                    }
                    other_char => return_page.push(other_char),
                }
            }
            if cachable {
                self.components.remove(component.as_str());
                self.cache.insert(component, return_page.clone());
            }
            return_page
        } else {
            eprintln!(
                "components: {:?}, cache: {:?}, input: {:?}",
                self.components, self.cache, component
            );
            panic!("This is a very bad situation.")
        };
    }
}

//  Read file -> Tree -> Render
// * Take arguements -> Parse file -> Return html
