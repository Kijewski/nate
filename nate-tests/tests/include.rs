use std::fmt::{Result, Write};

use nate::Nate;

#[test]
fn test_hello_world() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/include.html")]
    struct Template;

    let mut buf = String::new();
    write!(buf, "{}", Template)?;
    assert_eq!(buf, "<1><2><3></2></1>");
    Ok(())
}
