use std::fmt::{self, Result, Write};

use nate::Nate;

#[test]
fn test_lifetimes() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/greeting.html")]
    struct Template<'a> {
        user: &'a str,
    }

    let mut buf = String::new();
    write!(buf, "{}", Template { user: "<World>" })?;
    assert_eq!(buf, "<h1>Hello, &#60;World&#62;!</h1>");
    Ok(())
}

#[test]
fn test_nesting() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/greeting.html")]
    struct Template1<'a> {
        user: &'a str,
    }

    #[derive(Nate)]
    #[template(path = "templates/greeting-raw.html")]
    struct Template2<T: fmt::Display> {
        user: T,
    }

    let mut buf = String::new();
    let tmpl1 = Template1 { user: "<World>" };
    let tmpl2 = Template2 { user: &tmpl1 };
    write!(buf, "{}", tmpl2)?;
    assert_eq!(buf, "<h1>Hello, <h1>Hello, &#60;World&#62;!</h1>!</h1>");
    Ok(())
}

#[test]
fn test_nesting_where() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/greeting.html")]
    struct Template1<'a> {
        user: &'a str,
    }

    #[derive(Nate)]
    #[template(path = "templates/greeting-raw.html")]
    struct Template2<T>
    where
        T: fmt::Display,
    {
        user: T,
    }

    let mut buf = String::new();
    let tmpl1 = Template1 { user: "<World>" };
    let tmpl2 = Template2 { user: &tmpl1 };
    write!(buf, "{}", tmpl2)?;
    assert_eq!(buf, "<h1>Hello, <h1>Hello, &#60;World&#62;!</h1>!</h1>");
    Ok(())
}
