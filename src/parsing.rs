use crate::{
    ast::{Dom, Element, ElementType},
    LazyDom,
};
use std::{borrow::Cow, collections::HashMap, path::Component, str::Chars};

// WARN: REFACTOR ASAP. THIS CODE IS AWFUL AND UNREADABLE.
//       FAILURE TO DO SO WILL KILL THE PROJECT

// TODO:
// - [X] Core parsing
// - [ ] Error handling *You aren't ever doing this you fucking moron*
// - [ ] Fix bug where you can only parse one element
// - [ ] Variable passing (hx-params, slot)

fn parse_variable(char_iterator: &mut Chars, inputs: &HashMap<String, String>) -> String {
    let variable_name: String = char_iterator.take_while(|char| char != &'}').collect();
    inputs
        .get(variable_name.as_str())
        .map(|value| value.to_string())
        .unwrap_or_else(|| format!("{{{}}}", variable_name))
    //TODO: Maybe offer a flag for strict or relaxed parsing, allowing unretrievable errors to
    // return empty values
}

fn parse_tags(char_iterator: &mut Chars, inputs: &HashMap<String, String>) -> Vec<String> {
    if let Some(first_char) = char_iterator.next() {
        let mut in_quotes = false;
        let mut buffer = String::from(first_char);
        let mut collected_parts = Vec::new();

        // parses the first part of the element
        while let Some(char) = char_iterator.next() {
            match char {
                '"' | '\'' => in_quotes = !in_quotes,
                ' ' if !in_quotes && !buffer.is_empty() => {
                    collected_parts.push(buffer.clone());
                    buffer.clear()
                }
                // NOTE: an error could be thrown here in a stricter parsing mode if this isnt present
                '>' if !in_quotes => {
                    if !buffer.is_empty() {
                        collected_parts.push(buffer.clone());
                        buffer.clear();
                    }
                    break;
                }
                '{' => buffer.push_str(&parse_variable(char_iterator, inputs)),
                other_char => buffer.push(other_char),
            }
        }

        collected_parts
    } else {
        vec!["<".to_string()]
    }
}

pub(crate) fn parse(component: &str, components: &HashMap<String, LazyDom>) -> Result<(), ()> {
    let mut stack = vec![];

    match components.get(component) {
        Some(LazyDom::Parsed(dom)) => return Ok(()),
        Some(LazyDom::Unparsed(page)) => stack.push((component.to_string(), page.chars(), inputs)),
        None => return Err(()),
    };

    let mut buffer = String::new();

    while let Some((component_name, char_iterator, inputs)) = stack.pop() {
        let mut dom = Dom::new(component_name);

        while let Some(char) = char_iterator.next() {
            match char {
                '{' => {
                    let variable_data = parse_variable(&mut char_iterator, &inputs);

                    if element_stack.is_empty() {
                        return_page.push_str(&variable_data);
                    } else {
                        buffer.push_str(&variable_data);
                    }
                }
                '<' => {
                    if let Some(last_element) = element_stack.last_mut() {
                        match last_element.value {
                            ParsedState::Parsed(ref mut value) => {
                                value.push_str(&std::mem::take(&mut buffer))
                            }
                            ParsedState::Unparsed => {
                                last_element.value =
                                    ParsedState::Parsed(std::mem::take(&mut buffer))
                            }
                        };
                    };
                    let collected_parts = parse_tags(&mut char_iterator, &inputs);

                    if collected_parts[0].chars().next().unwrap().is_uppercase() {
                        let result = parse(
                            &collected_parts[0].to_string(),
                            inputs.clone(),
                            parsing_event_callback,
                        );
                        // TODO: Map parent here if its none.
                        //result.map_err(|err| if err.kind())

                        return_page.push_str(&result?);
                    }
                    // TODO: We can remove this check and instead just get the last element
                    else if let Some(last_element) = element_stack.last() {
                        if collected_parts[0] == format!("/{}", last_element.name) {
                            if let Some(mut element) = element_stack.pop() {
                                match element.value {
                                    ParsedState::Parsed(ref mut value) => {
                                        value.push_str(&std::mem::take(&mut buffer))
                                    }
                                    ParsedState::Unparsed => {
                                        element.value =
                                            ParsedState::Parsed(std::mem::take(&mut buffer))
                                    }
                                };

                                if element_stack.is_empty() {
                                    if !element.dropping {
                                        return_page.push_str(&element.serialise());
                                    }
                                } else {
                                    if !element.dropping {
                                        buffer.push_str(&element.serialise());
                                    }
                                }
                            }
                        }
                    } else {
                        let mut element = Element {
                            name: collected_parts[0].to_string(),
                            attributes: HashMap::from_iter(
                                collected_parts
                                    .iter()
                                    .skip(1)
                                    .filter(|attribute| attribute != &"/")
                                    .map(|attribute| {
                                        // Takes Element attributes and splits them by the =. If they dont
                                        // contain a = then it just takes the attribute name.
                                        // E.g. <button disabled command='toggle_popover'>...
                                        // The disabled will become ("disabled", "disabled") and the command
                                        // will become ("command", "toggle_popover")
                                        let (key, value) = attribute
                                            .split_once('=')
                                            .unwrap_or((attribute, attribute));
                                        (key.to_string(), value.to_string())
                                    }),
                            ),
                            ..Default::default()
                        };

                        if let Some(event_result) = parsing_event_callback(element.clone()) {
                            element_stack.push(event_result);
                        } else {
                            element.dropping = true;
                            element_stack.push(element);
                        }
                    }
                }

                other_char => {
                    if element_stack.is_empty() {
                        return_page.push(other_char);
                    } else {
                        buffer.push(other_char);
                    }
                }
            }
        }
    }

    Ok(())
}
