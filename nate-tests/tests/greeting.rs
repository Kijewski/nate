use std::cell;
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

#[test]
fn test_int() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/greeting.html")]
    struct Template {
        user: i32,
    }

    let mut buf = String::new();
    write!(buf, "{}", Template { user: 4711 })?;
    assert_eq!(buf, "<h1>Hello, 4711!</h1>");
    Ok(())
}

#[test]
fn test_float() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/greeting.html")]
    struct Template {
        user: f64,
    }

    let mut buf = String::new();
    write!(buf, "{}", Template {
        user: 12300000000000000000000000000000000000000.0
    })?;
    assert_eq!(buf, "<h1>Hello, 1.23e40!</h1>");
    Ok(())
}

#[test]
fn test_cell() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/greeting.html")]
    struct Template<'a> {
        user: cell::Ref<'a, &'a str>,
    }

    let mut buf = String::new();
    write!(buf, "{}", Template {
        user: cell::RefCell::new("<WORLD>").borrow(),
    })?;
    assert_eq!(buf, "<h1>Hello, &#60;WORLD&#62;!</h1>");
    Ok(())
}

// This test would fail if the float specialization is not used.
#[test]
fn test_float_cell() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/greeting.html")]
    struct Template<'a> {
        user: cell::Ref<'a, f64>,
    }

    let mut buf = String::new();
    write!(buf, "{}", Template {
        user: cell::RefCell::new(12300000000000000000000000000000000000000.0).borrow(),
    })?;
    assert_eq!(buf, "<h1>Hello, 1.23e40!</h1>");
    Ok(())
}

#[test]
fn test_hello_string() -> Result {
    use nate::RenderInto;

    #[derive(Nate)]
    #[template(path = "templates/hello_world.html")]
    struct Template;

    let mut buf = String::new();
    Template.render_string(&mut buf)?;
    assert_eq!(buf, "Hello, World!");
    Ok(())
}
