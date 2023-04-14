#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[doc(hidden)]
pub use core;
use core::fmt;
#[cfg(feature = "alloc")]
use core::fmt::Write as _;
use core::marker::PhantomData;

pub use crate::escape::{EscapeKind, XmlEscape};
pub use crate::fast_float::FloatKind;
pub use crate::fast_integer::IntKind;
pub use crate::raw::RawKind;

/// [Zero sized](https://doc.rust-lang.org/1.56.0/nomicon/exotic-sizes.html#zero-sized-types-zsts)
/// wrapper used to select an escape function
///
/// To implement your own specialization, you can implement your own trait this way:
///
/// ```rust,ignore
/// use std::fmt;
///
/// // First you have to add a "marker" trait for types that you want to escape
/// // with your custom escaper.
///
/// trait MyEscapeMarker {}
///
/// impl<T: MyEscapeMarker> MyEscapeMarker for &T {}
///
/// // You can implement your custom escaper for multiple types.
///
/// enum TerribleXml<'a> {
///     Start(&'a str),
///     End(&'a str),
///     Text(&'a str),
/// }
///
/// impl MyEscapeMarker for TerribleXml<'_> {}
///
/// // Second you add a new trait that wraps a reference to the value to escape.
/// // If the value is `Copy`, then you don't have to keep reference to `value`.
/// // You must not capture a reference to `self`, because `self` is ephemeral.
///
/// trait MyEscapeKind {
///     #[inline]
///     fn wrap<'a>(&self, value: &'a TerribleXml) -> MyEscaper<'a> {
///         MyEscaper { value }
///     }
/// }
///
/// impl<T: MyEscapeMarker> MyEscapeKind for nate::EscapeWrapper<T> {}
///
/// // Lastly you have to implement `std::fmt::Display` for your escaper.
///
/// struct MyEscaper<'a> {
///     value: &'a TerribleXml<'a>,
/// }
///
/// impl fmt::Display for MyEscaper<'_> {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         match self.value {
///             TerribleXml::Start(tag) => write!(f, "<{}>", tag),
///             TerribleXml::End(tag) => write!(f, "</{}>", tag),
///             TerribleXml::Text(text) => f.write_str(text),
///         }
///     }
/// }
///
/// // Then you can use the escaper in your templates.
/// // The trait `MyEscapeKind` has to be in scope of the template declaration.
///
/// #[derive(nate::Nate)]
/// #[template(path = "templates/custom-escaper.html")]
/// struct Template<'a> {
///     elems: &'a [TerribleXml<'a>],
/// }
///
/// #[test]
/// fn test_custom_escaper() {
///     let template = Template { elems: &[
///         TerribleXml::Text("Hello, "),
///         TerribleXml::Start("strong"),
///         TerribleXml::Text("world"),
///         TerribleXml::End("b"),
///         TerribleXml::Text("!"),
///     ] };
///     let data = format!("{}", template);
///     assert_eq!(data, "Hello, <strong>world</b>!");
/// }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct EscapeWrapper<E>(PhantomData<E>);

impl<E> EscapeWrapper<E> {
    #[doc(hidden)]
    #[inline]
    pub fn new(_: &E) -> Self {
        Self(PhantomData)
    }
}

#[doc(hidden)]
pub trait WriteAny {
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> fmt::Result;
    fn write_str(&mut self, s: &str) -> fmt::Result;
}

#[cfg(feature = "std")]
pub(crate) struct WriteIo<W: std::io::Write>(pub(crate) W);

pub(crate) struct WriteFmt<W: fmt::Write>(pub(crate) W);

#[cfg(feature = "alloc")]
pub(crate) struct WriteString<'a>(pub(crate) &'a mut alloc::string::String);

#[cfg(feature = "std")]
impl<W: std::io::Write> WriteAny for WriteIo<W> {
    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> fmt::Result {
        match <W as std::io::Write>::write_fmt(&mut self.0, fmt) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_fmt(format_args!("{}", s))
    }
}

impl<W: fmt::Write> WriteAny for WriteFmt<W> {
    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> fmt::Result {
        <W as fmt::Write>::write_fmt(&mut self.0, fmt)
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        <W as fmt::Write>::write_str(&mut self.0, s)
    }
}

#[cfg(feature = "alloc")]
impl WriteAny for WriteString<'_> {
    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> fmt::Result {
        if let Some(s) = fmt.as_str() {
            self.0.push_str(s);
            Ok(())
        } else {
            self.0.write_fmt(fmt)
        }
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.push_str(s);
        Ok(())
    }
}

/// Optimized trait methods to render a NaTE template
///
/// Every NaTE template implements this trait.
pub trait RenderInto {
    #[doc(hidden)]
    fn render_into(&self, output: impl WriteAny) -> fmt::Result;

    /// Render the output into an [`fmt::Write`](std::fmt::Write) object
    #[inline]
    fn render_fmt(&self, output: impl fmt::Write) -> fmt::Result {
        self.render_into(WriteFmt(output))
    }

    /// Render the output into an [`io::Write`](std::io::Write) object
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    #[inline]
    fn render_io(&self, output: impl std::io::Write) -> fmt::Result {
        self.render_into(WriteIo(output))
    }

    /// Render the output into a [`String`]
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[inline]
    fn render_string(&self, output: &mut alloc::string::String) -> fmt::Result {
        self.render_into(WriteString(output))
    }
}
