use std::fmt::{Result, Write};

use nate::Nate;

#[test]
fn test_hello_world() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/hello_world.html")]
    struct Template;

    let mut buf = String::new();
    write!(buf, "{}", Template)?;
    assert_eq!(buf, "Hello, World!");
    Ok(())
}

#[test]
fn test_greeting() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/greeting.html")]
    struct Template {
        user: String,
    }

    let mut buf = String::new();
    write!(buf, "{}", Template {
        user: "<World>".to_owned()
    })?;
    assert_eq!(buf, "<h1>Hello, &#60;World&#62;!</h1>");
    Ok(())
}

#[test]
fn test_greeting_raw() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/greeting-raw.html")]
    struct Template {
        user: String,
    }

    let mut buf = String::new();
    write!(buf, "{}", Template {
        user: "<World>".to_owned()
    })?;
    assert_eq!(buf, "<h1>Hello, <World>!</h1>");
    Ok(())
}
