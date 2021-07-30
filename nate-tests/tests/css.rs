use std::fmt::{Result, Write};

use nate::Nate;

#[test]
fn test_css() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/test.css")]
    struct Template<'a> {
        color: &'a str,
        background: &'a str,
    }

    let tmpl = Template {
        color: "red",
        background: "lightyellow",
    };
    let mut buf = String::new();
    write!(buf, "{}", tmpl)?;
    assert_eq!(
        buf,
        r#"#body {
    color: red;
    background: lightyellow;
}"#
    );
    Ok(())
}
