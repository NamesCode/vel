use crate::ast::{Element, ParsedState};

impl Element {
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
