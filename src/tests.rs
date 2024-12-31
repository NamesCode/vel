#[cfg(test)]
pub mod tests {
    use crate::VelInstance;
    use std::collections::HashMap;

    #[test]
    //fn render_only<'a>() -> Result<(), RenderingError<'a>> {
    fn render_only<'a>() -> Result<(), crate::Error> {
        let test_instance = VelInstance::new(HashMap::from([(
            "Component".to_string(),
            r#"
        <div test>
            <span style='color: red'>this is the inside text</span>, I sure do hope nothing happens to it
        </div>
        <p>hello, world</p>"#.to_string(),
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
    fn render_and_filter<'a>() -> Result<(), crate::Error> {
        let test_instance = VelInstance::new(HashMap::from([(
            "Component".to_string(),
            r#"
        <div test>
            <span style='color: red'>this is the inside text</span>, I sure do hope nothing happens to it
        </div>
        <p>hello, world</p>"#.to_string(),
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
