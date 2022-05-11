use std::fmt::{Result, Write};
use std::num::NonZeroU64;

use nate::Nate;

#[test]
fn test_nonzero() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/escaping.html")]
    struct Template {
        data: NonZeroU64,
    }

    let tmpl = Template {
        data: NonZeroU64::new(42).unwrap(),
    };
    let mut buf = String::new();
    write!(buf, "{}", tmpl)?;
    assert_eq!(buf, "* 42\n* 42\n* 42\n* 42\n");
    Ok(())
}
