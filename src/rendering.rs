// SPDX-FileCopyrightText: 2025 Name <lasagna@garfunkle.space>
//
// SPDX-License-Identifier: EUPL-1.2

//use crate::ast::Element;

//impl Element {
//    //pub fn serialise(self) -> Result<>
//    pub fn serialise(&self) -> String {
//        let text = match &self.value {
//            ParsedState::Parsed(text) => &text,
//            ParsedState::Unparsed => &String::new(),
//        };
//
//        let attributes: String = self
//            .attributes
//            .iter()
//            .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
//            .collect();
//
//        format!("<{0}{attributes}>{text}</{0}>", self.name)
//    }
//}
