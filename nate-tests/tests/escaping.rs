use std::fmt::{Debug, Display, Result, Write};

use nate::Nate;

#[test]
fn test_css() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/escaping.html")]
    struct Template<Data: Debug + Display> {
        data: Data,
    }

    let tmpl = Template {
        data: r#"Test & <Test> & "test" & 'test'"#,
    };
    let mut buf = String::new();
    write!(buf, "{}", tmpl)?;
    assert_eq!(
        buf,
        "\
* Test &#38; &#60;Test&#62; &#38; &#34;test&#34; &#38; &#39;test&#39;\n\
* Test & <Test> & \"test\" & 'test'\n\
* &#34;Test &#38; &#60;Test&#62; &#38; \\&#34;test\\&#34; &#38; &#39;test&#39;&#34;\n\
* &#34;Test &#38; &#60;Test&#62; &#38; \\&#34;test\\&#34; &#38; &#39;test&#39;&#34;\n"
    );
    Ok(())
}
