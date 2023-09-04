use std::fmt::{Result, Write};

use nate::Nate;

#[test]
fn closing_brace_in_data() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/closing-brace-in-data.html")]
    struct Template<'a> {
        data: &'a str,
    }

    let mut buf = String::new();
    let tmpl = Template {
        data: "Test äö\\ü"
    };
    write!(buf, "{}", tmpl)?;
    assert_eq!(buf, "&#34;Test äö\\\\ü&#34;\n");
    Ok(())
}

/// <https://github.com/Kijewski/nate/issues/52>
#[test]
fn clippy_pedantic_should_be_fine() {
    #![deny(clippy::pedantic)]

    #[derive(Nate)]
    #[template(path = "templates/99-bottles.html")]
    struct Template {
        limit: usize,
    }
}
