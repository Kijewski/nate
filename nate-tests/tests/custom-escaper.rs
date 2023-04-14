use std::fmt;

use nate::Nate;

// First you have to add a "marker" trait for types that you want to escape
// with your custom escaper.

trait MyEscapeMarker {}

impl<T: MyEscapeMarker> MyEscapeMarker for &T {}

// You can implement your custom escaper for multiple types.

enum TerribleXml<'a> {
    Start(&'a str),
    End(&'a str),
    Text(&'a str),
}

impl MyEscapeMarker for TerribleXml<'_> {}

// Second you add a new trait that wraps a reference to the value to escape.
// If the value is `Copy`, then you don't have to keep reference to `value`.
// You must not capture a reference to `self`, because `self` is ephemeral.

trait MyEscapeKind {
    #[inline]
    fn wrap<'a>(&self, value: &'a TerribleXml) -> MyEscaper<'a> {
        MyEscaper { value }
    }
}

impl<T: MyEscapeMarker> MyEscapeKind for nate::EscapeWrapper<T> {}

// Lastly you have to implement `std::fmt::Display` for your escaper.

struct MyEscaper<'a> {
    value: &'a TerribleXml<'a>,
}

impl fmt::Display for MyEscaper<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            TerribleXml::Start(tag) => write!(f, "<{}>", tag),
            TerribleXml::End(tag) => write!(f, "</{}>", tag),
            TerribleXml::Text(text) => f.write_str(text),
        }
    }
}

// Then you can use the escaper in your templates.
// The trait `MyEscapeKind` has to be in scope of the template declaration.

#[derive(Nate)]
#[template(path = "templates/custom-escaper.html")]
struct Template<'a> {
    elems: &'a [TerribleXml<'a>],
}

#[test]
fn test_custom_escaper() {
    let template = Template {
        elems: &[
            TerribleXml::Text("Hello, "),
            TerribleXml::Start("strong"),
            TerribleXml::Text("world"),
            TerribleXml::End("b"),
            TerribleXml::Text("!"),
        ],
    };
    let data = format!("{}", template);
    assert_eq!(data, "Hello, <strong>world</b>!");
}
