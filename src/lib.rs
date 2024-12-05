use std::{
    collections::HashMap,
    fmt::{self, Display},
    str::Chars,
};

#[derive(Debug)]
pub struct VelInstance<'a> {
    components: HashMap<&'a str, &'a str>,
}

impl Display for VelInstance<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "hamburger")
    }
}

#[derive(Debug)]
pub enum ParsedState<T> {
    Unparsed,
    Parsed(T),
}

impl<T> Default for ParsedState<T> {
    fn default() -> ParsedState<T> {
        ParsedState::Unparsed
    }
}

impl<T> ParsedState<T> {
    fn is_parsed(&self) -> bool {
        match self {
            ParsedState::Parsed(_) => true,
            ParsedState::Unparsed => false,
        }
    }
}

#[derive(Default, Debug)]
pub struct Element {
    /// The element name; e.g. <p> = "p"
    pub name: String,
    /// The elements attributes; e.g. <p style='color: red'> = ("style", "color: red")
    pub attributes: HashMap<String, String>,
    /// The elements contained value, if it is None then that just means it has not been parsed
    /// yet.
    pub value: ParsedState<String>,
}

impl Element {
    pub fn new() -> Self {
        Element::default()
    }

    //pub fn serialise(self) -> Result<>
    pub fn serialise(self) -> String {
        if let ParsedState::Parsed(text) = self.value {
            let mut attributes = String::new();
            for (attribute, value) in self.attributes.iter() {
                attributes = format!("{attributes} {attribute}={value}")
            }

            format!("<{0}{attributes}>{text}</{0}>", self.name)
        } else {
            //Err("THE TEXT IS UNPARSED, CANNOT BE CALLED HERE")
            String::new()
        }
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

    pub fn render<F>(
        &'_ mut self,
        component: String,
        inputs: HashMap<&str, &str>,
        parsing_event_callback: F,
    ) -> String
    where
        F: Fn(Element) -> Option<Element> + std::marker::Copy,
    {
        fn parse_variable(char_iterator: &mut Chars, inputs: &HashMap<&str, &str>) -> String {
            let variable_name: String = char_iterator.take_while(|char| char != &'}').collect();
            inputs
                .get(variable_name.as_str())
                .map(|value| value.to_string())
                .unwrap_or_else(|| format!("{{{}}}", variable_name))
        }

        fn parse_tags(char_iterator: &mut Chars, inputs: &HashMap<&str, &str>) -> Vec<String> {
            if let Some(first_char) = char_iterator.next() {
                let mut in_quotes = false;
                let mut buffer = String::from(first_char);
                let mut collected_parts = Vec::new();

                // parses the first part of the element
                while let Some(char) = char_iterator.next() {
                    // NOTE: Whilst this is more complex than it needs to be, it gives us
                    // the opportunity to do things like Svelte's slot in future.
                    match char {
                        '"' | '\'' => in_quotes = !in_quotes,
                        ' ' => {
                            if !in_quotes && !buffer.is_empty() {
                                collected_parts.push(buffer.clone());
                                buffer.clear()
                            }
                        }
                        '>' => {
                            if !in_quotes {
                                if !buffer.is_empty() {
                                    collected_parts.push(buffer.clone());
                                    buffer.clear();
                                }
                                break;
                            }
                        }
                        '{' => buffer.push_str(&parse_variable(char_iterator, inputs)),
                        //'<' => {
                        //    if !in_quotes {
                        //        collected_parts.push(buffer.clone());
                        //        buffer.clear();
                        //
                        //        let tag_value = parse_tags(parent, char_iterator, inputs);
                        //        if tag_value.contains(format!("/{}", first_char).as_str()) {
                        //            break;
                        //        } else {
                        //            collected_parts.push(tag_value);
                        //            buffer.clear();
                        //        }
                        //    }
                        //}
                        other_char => buffer.push(other_char),
                    }
                }

                println!("attributes: {:?}", collected_parts);
                collected_parts
            } else {
                vec!["<".to_string()]
            }
        }

        if let Some(page) = self.components.get(component.as_str()) {
            let mut return_page = String::with_capacity(page.len());
            let mut char_iterator = page.chars();

            while let Some(char) = char_iterator.next() {
                match char {
                    '{' => {
                        let variable_data = parse_variable(&mut char_iterator, &inputs);
                        return_page.push_str(&variable_data);
                    }
                    other_char => return_page.push(other_char),
                    '<' => {
                        let element_parts = parse_tags(&mut char_iterator, &inputs);

                        let mut element = Element {
                            name: element_parts[0],
                            attributes: HashMap::from_iter(
                                element_parts
                                    .iter()
                                    .skip(1)
                                    .take_while(|attribute| !attribute.is_empty())
                                    .map(|attribute| {
                                        // Takes Element attributes and splits them by the =. If they dont
                                        // contain a = then it just takes the attribute name.
                                        // E.g. <button disabled command='toggle_popover'>...
                                        // The disabled will become ("disabled", "disabled") and the command
                                        // will become ("command", "toggle_popover")
                                        let (key, value) = attribute
                                            .split_once('=')
                                            .unwrap_or((attribute, attribute));
                                        (key.to_owned(), value.to_owned())
                                    }),
                            ),
                            value: ParsedState::Unparsed,
                        };

                        if let Some(event_result) = parsing_event_callback(element) {
                            element = event_result
                        } else {
                            return "".to_string();
                        }

                        //// NOTE: We have to check this here as the user may have, for some reason, set the
                        //// value of the element using the closure.
                        if !element.value.is_parsed() {
                            let mut buffer = String::new();
                            let mut in_quotes = false;

                            while let Some(char) = char_iterator.next() {
                                match char {
                                    '"' | '\'' => in_quotes = !in_quotes,
                                    '{' => buffer
                                        .push_str(&parse_variable(&mut char_iterator, &inputs)),
                                    '<' => {
                                        if !in_quotes {
                                            collected_parts.push(buffer.clone());
                                            buffer.clear();
                                    
                                            let tag_value = parse_tags(parent, char_iterator, inputs);
                                            if tag_value.contains(format!("/{}", first_char).as_str()) {
                                                break;
                                            } else {
                                                collected_parts.push(tag_value);
                                                buffer.clear();
                                            }
                                        }
                                    }
                                    other_char => buffer.push(other_char),
                                }
                            }
                        }

                        element.serialise();

                        //if collected_parts[0].starts_with(|first_char: char| first_char.is_uppercase()) {
                        //    parent.render(
                        //        collected_parts[0].to_owned(),
                        //        inputs,
                        //        parsing_event_callback,
                        //    )
                        //} else {
                        //    "shart".to_string()
                        //}
                    }
                }
            }

            return_page
        } else {
            eprintln!("components: {:?}, input: {:?}", self.components, component);
            panic!("This is a very bad situation. The component probably does NOT exist.")
        }
    }
}

//   Read file -> Tree -> Render
// * Take arguements -> Parse file -> Return html
