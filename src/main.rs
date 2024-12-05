use std::{collections::HashMap, fs::read_to_string};
use vel::{Element, ParsedState, VelInstance};

fn main() {
    let file = read_to_string("./src/test.html").unwrap();
    let file = read_to_string("./src/test2.html").unwrap();
    let mut test = VelInstance::new(HashMap::from([("t", file.as_str())]));
    let result = test.render("t".to_owned(), HashMap::new(), |element| {
        Some(Element {
            name: "bs".to_string(),
            attributes: HashMap::new(),
            value: ParsedState::Parsed("balls".to_string()),
        })
    });
    dbg!(result);
}
