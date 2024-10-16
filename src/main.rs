use std::collections::HashMap;

mod vel {
    use core::fmt;
    use std::{collections::HashMap, fmt::Display, thread::panicking};

    #[derive(Debug)]
    pub struct VelInstance<'a> {
        components: HashMap<&'a str, &'a str>,
        cache: HashMap<&'a str, String>,
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

        pub fn render<'b>(
            &'b mut self,
            component: &'a str,
            inputs: HashMap<&'a str, &'a str>,
            //) -> Result<&Self, Err> {
        ) -> String {
            fn parse_variable(input: &str) -> Option<(usize, &str)> {
                for (index, char) in input.char_indices() {
                    if char == '}' {
                        return Some((index + 2, &input[..index].trim()));
                    } else if !char.is_alphabetic() && char != ' ' {
                        println!("{}", char);
                        break;
                    };
                }
                return None;
            }

            fn parse_component<'b>(
                input: &str,
                components: &HashMap<&'b str, &'b str>,
                inputs: HashMap<&'b str, &'b str>,
            ) -> Option<(usize, &'b str, HashMap<&'b str, &'b str>)> {
                let mut chars = input.char_indices();

                if let Some((_, char)) = chars.next() {
                    if char.is_lowercase() {
                        return None;
                    }
                } else {
                    return None;
                }

                if let Some((name_end_pos, _)) = chars
                    .clone()
                    .skip_while(|(_, char)| char.is_alphanumeric())
                    .next()
                {
                    let mut end_pos = 0;
                    let mut params: HashMap<&str, &str> = HashMap::new();

                    let mut name_start_index = 0; // For attribute names
                    let mut value_start_index = 0; // For attribute values
                    let mut in_quotes = false; // Track whether we're in quotes

                    for (index, char) in chars {
                        match char {
                            '/' | '>' => {
                                // End of component detected
                                if !in_quotes {
                                    end_pos = index;
                                    break;
                                }
                            }
                            '"' | '\'' => {
                                // Toggle in_quotes state
                                if in_quotes {
                                    // Ending the value
                                    let key = &input[name_start_index..index].trim();
                                    let value = &input[value_start_index..index].trim();
                                    params.insert(key, value);
                                    value_start_index = 0; // Reset for the next attribute
                                    in_quotes = false; // Reset quotes
                                } else {
                                    // Starting a new quoted value
                                    value_start_index = index + 1; // Move past the quote
                                    in_quotes = true;
                                }
                            }
                            ' ' => {
                                // Handle spaces
                                if !in_quotes {
                                    if name_start_index != 0 {
                                        // If we finished reading an attribute name
                                        let key = &input[name_start_index..index].trim();
                                        // If no value was provided, treat it as a boolean attribute
                                        params.insert(key, key);
                                        name_start_index = 0; // Reset for the next attribute
                                    }
                                }
                            }
                            char => {
                                // Capture alphanumeric characters for attribute names
                                if char.is_alphanumeric() {
                                    if name_start_index == 0 {
                                        name_start_index = index; // Start of an attribute name
                                    }
                                }
                            }
                        }
                    }

                    if let Some(name) = components.get(&input[..name_end_pos]) {
                        return Some((end_pos, name.clone(), inputs));
                    }
                }

                return None;
            }

            // HACK: Lazy fuck couldnt make a proper implementation because they suck ass at
            // borrowing. FIX THIS!!
            return if let Some(cached_page) = self.cache.get(component) {
                cached_page.to_string()
            } else if let Some(page) = self.components.get(component) {
                let mut page = page.to_string();
                let mut last_pos = 0;
                let mut cachable = true;

                while last_pos + 1 < page.len() {
                    match page[last_pos..last_pos + 1].chars().next().unwrap() {
                        '{' => {
                            cachable = false;
                            if let Some((close_brack_pos, var_name)) =
                                parse_variable(&page[last_pos + 1..])
                            {
                                if let Some(value) = &inputs.get(var_name) {
                                    page.replace_range(last_pos..last_pos + close_brack_pos, value);
                                    last_pos += 1 + value.len()
                                } else {
                                    last_pos += 1 + close_brack_pos
                                }
                            } else {
                                last_pos += 1
                            }
                        }
                        '<' => {
                            cachable = false;
                            if let Some((index, component, inputs)) = parse_component(
                                &page[last_pos + 1..],
                                &self.components,
                                inputs.clone(),
                            ) {
                                let _ = self.render(component, inputs);
                            } else {
                                last_pos += 1
                            }
                        }
                        _ => last_pos += 1,
                    }
                    if cachable {
                        self.components.remove(component);
                        self.cache.insert(component, page.clone());
                    }
                }
                page
            } else {
                eprintln!(
                    "components: {:?}, cache: {:?}, input: {:?}",
                    self.components, self.cache, component
                );
                panic!("This is a very bad situation.")
            };
        }
    }
}

use crate::vel::VelInstance;

fn main() {
    let mut test = VelInstance::new(HashMap::from([
        ("Component", "banana {data} {data} {data} <Ab/>"),
        ("Ab", "Bollocks"),
    ]));
    //dbg!(&test);
    let test_output = test.render("Component", HashMap::from([("data", "cheese")]));
    dbg!(&test_output);
}

//  Read file -> Tree -> Render
// * Take arguements -> Parse file -> Return html
