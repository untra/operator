pub(crate) fn render(template: &str, values: &[(&str, &str)]) -> String {
    let mut parts = template.split("__OP_");
    let mut script = parts.next().unwrap_or_default().to_owned();
    for part in parts {
        let (name, rest) = part
            .split_once("__")
            .expect("unterminated script placeholder");
        let (_, value) = values
            .iter()
            .find(|(key, _)| *key == name)
            .unwrap_or_else(|| panic!("missing script placeholder {name}"));
        script.push_str(value);
        script.push_str(rest);
    }
    script
}

pub(crate) fn body(script: &str) -> &str {
    script.split_once('\n').expect("script shebang").1
}

#[cfg(test)]
mod tests {
    use super::render;

    #[test]
    fn test_render_preserves_placeholder_text_in_input() {
        assert_eq!(
            render(
                "x=__OP_VALUE__\ny=__OP_OTHER__\n",
                &[("VALUE", "'__OP_OTHER__'"), ("OTHER", "7")]
            ),
            "x='__OP_OTHER__'\ny=7\n"
        );
    }
}
