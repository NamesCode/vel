use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Debug, Display},
    str::Chars,
};

#[derive(Debug, Clone, Copy)]
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
    pub fn is_parsed(&self) -> bool {
        match self {
            ParsedState::Parsed(_) => true,
            ParsedState::Unparsed => false,
        }
    }
}

pub enum ParsingError {
    UnclosedTag {
        value: Vec<String>,
        //line: usize,
        //char: usize,
    },
}

#[derive(Debug, Clone)]
//pub enum RenderingError<'a> {
pub enum RenderingError {
    UnknownComponent {
        name: String,
        //instance_components: HashMap<&'a str, &'a str>,
    },
    UnknownInput {
        name: String,
        //inputs: HashMap<&'a str, &'a str>,
    },
    HitRecursionLimit {
        component: String,
        parent_component: Option<String>,
    },
}

//impl Display for RenderingError<'_> {
impl Display for RenderingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            //Self::UnknownComponent { name, instance_components } => write!(f, "Component not found in the instance components. Unknown component: {name}, Instance components: {:?}.", instance_components),
            //Self::UnknownInput { name, inputs } => write!(f, "Input not found in the provided inputs. Unknown input: {name}, inputs: {:?}.", inputs),
            Self::UnknownComponent { name } => write!(f, "Component not found in the instance components. Unknown component: {name}."),
            Self::UnknownInput { name } => write!(f, "Input not found in the provided inputs. Unknown input: {name}."),
            Self::HitRecursionLimit { component, parent_component } => write!(f, "The recursion limit has been hit while rendering '{component}'. Parent component: {:?}", parent_component),
        }
    }
}

//impl Error for RenderingError<'_> {
impl Error for RenderingError {
    fn description(&self) -> &str {
        todo!()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Element {
    /// The element name; e.g. <p> = "p"
    pub name: String,
    /// The elements attributes; e.g. <p style='color: red'> = ("style", "color: red")
    pub attributes: HashMap<String, String>,
    /// The elements contained value, if it is None then that just means it has not been parsed
    /// yet.
    pub value: ParsedState<String>,
    dropping: bool,
}

impl Element {
    pub fn new() -> Self {
        Element::default()
    }

    //pub fn serialise(self) -> Result<>
    pub fn serialise(&self) -> String {
        let text = match &self.value {
            ParsedState::Parsed(text) => &text,
            ParsedState::Unparsed => &String::new(),
        };

        let attributes: String = self
            .attributes
            .iter()
            .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
            .collect();

        format!("<{0}{attributes}>{text}</{0}>", self.name)
    }
}

#[derive(Debug)]
pub struct VelInstance<'a> {
    components: HashMap<&'a str, &'a str>,
    recursion_limit: u8,
}

impl<'a> VelInstance<'a> {
    pub fn new(components: HashMap<&'a str, &'a str>) -> Self {
        Self {
            components,
            recursion_limit: 15,
        }
    }

    pub fn set_recursion_limit(&mut self, recusion_limit: u8) -> &mut Self {
        self.recursion_limit = recusion_limit;
        self
    }

    pub fn extend(&mut self, components: HashMap<&'a str, &'a str>) -> &mut Self {
        self.components.extend(components);
        self
    }

    pub fn render<F>(
        &'a self,
        component: String,
        inputs: HashMap<&str, &str>,
        parsing_event_callback: F,
        //) -> Result<String, RenderingError<'a>>
    ) -> Result<String, RenderingError>
    where
        F: Fn(Element) -> Option<Element> + std::marker::Copy,
    {
        let recursion_depth = self.recursion_limit;
        render_recursively(
            self,
            component,
            inputs,
            recursion_depth,
            parsing_event_callback,
        )
    }
}

// TODO:
// - [X] Core parsing
// - [ ] Error handling
// - [ ] Refactor
// - [ ] Variable passing (hx-params, slot)

fn parse_variable(char_iterator: &mut Chars, inputs: &HashMap<&str, &str>) -> String {
    let variable_name: String = char_iterator.take_while(|char| char != &'}').collect();
    inputs
        .get(variable_name.as_str())
        .map(|value| value.to_string())
        .unwrap_or_else(|| format!("{{{}}}", variable_name))
    // NOTE: Maybe offer a flag for strict or relaxed parsing, allowing unretrievable errors to
    // return empty values
}

fn parse_tags(char_iterator: &mut Chars, inputs: &HashMap<&str, &str>) -> Vec<String> {
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
                '>' if !in_quotes => {
                    if !buffer.is_empty() {
                        collected_parts.push(buffer.clone());
                        buffer.clear();
                    }
                    break;
                } // NOTE: an error could be thrown here in a stricter parsing mode if this isnt
                // present
                '{' => buffer.push_str(&parse_variable(char_iterator, inputs)),
                other_char => buffer.push(other_char),
            }
        }

        collected_parts
    } else {
        vec!["<".to_string()]
    }
}

fn render_recursively<'a, F>(
    parent: &'a VelInstance,
    component: String,
    inputs: HashMap<&str, &str>,
    recursion_depth: u8,
    parsing_event_callback: F,
    //) -> Result<String, RenderingError<'a>>
) -> Result<String, RenderingError>
where
    F: Fn(Element) -> Option<Element> + std::marker::Copy,
{
    if recursion_depth == 0 {
        return Err(RenderingError::HitRecursionLimit {
            component,
            parent_component: None,
        });
    }

    if let Some(page) = parent.components.get(component.as_str()) {
        let mut return_page = String::new();
        let mut buffer = String::new();

        let mut char_iterator = page.chars();
        let mut element_stack: Vec<Element> = vec![];

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
                        let mut result = render_recursively(
                            parent,
                            collected_parts[0].to_string(),
                            inputs.clone(),
                            recursion_depth - 1,
                            parsing_event_callback,
                        );
                        // TODO: Map parent here if its none.
                        //result.map_err(|err| if err.kind())

                        return_page.push_str(&result?);
                    }
                    // TODO: We can remove this check and instead just get the last element
                    else if !element_stack.is_empty() && collected_parts[0].contains('/') {
                        if collected_parts[0] == format!("/{}", element_stack.last().unwrap().name)
                        {
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

        Ok(return_page)
    } else {
        Err(RenderingError::UnknownComponent {
            name: component,
            //instance_components: parent.components.clone(),
        })
    }
}

//   Read file -> Tree -> Render
// * Take arguements -> Parse file -> Return html

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    //fn render_only<'a>() -> Result<(), RenderingError<'a>> {
    fn render_only<'a>() -> Result<(), RenderingError> {
        let test_instance = VelInstance::new(HashMap::from([(
            "Component",
            r#"
        <div test>
            <span style='color: red'>this is the inside text</span>, I sure do hope nothing happens to it
        </div>
        <p>hello, world</p>"#,
        )]));

        let result = test_instance.render("Component".to_string(), HashMap::new(), |element| {
            Some(element)
        })?;
        println!("render_only result: {}", result.trim());
        assert_eq!(
            result.trim(),
            r#"<div test="test">
            <span style="color: red">this is the inside text</span>, I sure do hope nothing happens to it
        </div>
        <p>hello, world</p>"#,
        );

        Ok(())
    }

    #[test]
    //fn render_and_filter<'a>() -> Result<(), RenderingError<'a>> {
    fn render_and_filter<'a>() -> Result<(), RenderingError> {
        let test_instance = VelInstance::new(HashMap::from([(
            "Component",
            r#"
        <div test>
            <span style='color: red'>this is the inside text</span>, I sure do hope nothing happens to it
        </div>
        <p>hello, world</p>"#,
        )]));

        let result = test_instance.render("Component".to_string(), HashMap::new(), |element| {
            if element.name != "div".to_string() {
                return Some(element);
            }
            None
        })?;

        println!("render_and_filter result: {}", result.trim());
        assert_eq!(result.trim(), "<p>hello, world</p>");
        Ok(())
    }
}
