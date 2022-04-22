#![no_implicit_prelude]

#[cfg(feature = "alloc")]
use super::details::alloc;
use super::details::std::fmt::{self, Write as _};
use super::details::std::prelude::v1::*;
use super::details::std::write;

#[doc(hidden)]
pub trait WriteAny {
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> fmt::Result;
}

/// Optimized trait methods to render a NaTE template
///
/// Every NaTE template implements this trait.
pub trait RenderInto {
    #[doc(hidden)]
    fn render_into(&self, output: impl WriteAny) -> fmt::Result;

    /// Render the output into an fmt::Write object
    #[inline]
    fn render_fmt(&self, output: impl fmt::Write) -> fmt::Result {
        self.render_into(super::details::WriteFmt(output))
    }

    /// Render the output into an io::Write object
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "alloc", feature = "std"))))]
    #[inline]
    fn render_io(&self, output: impl alloc::io::Write) -> fmt::Result {
        self.render_into(super::details::WriteIo(output))
    }

    /// Render the output into a new string
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "alloc", feature = "std"))))]
    fn render_string(&self) -> Result<String, fmt::Error> {
        let mut result = String::new();
        self.render_fmt(&mut result)?;
        Ok(result)
    }

    /// Render the output into a new vector
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "alloc", feature = "std"))))]
    fn render_bytes(&self) -> Result<Vec<u8>, fmt::Error> {
        let mut result = Vec::new();
        self.render_io(&mut result)?;
        Ok(result)
    }
}

/// A wrapper around a [displayable][fmt::Display] type that makes it write out XML escaped.
///
/// All characters are written as is except `"`, `&`, `'`, `<`, and `>` which are printed as e.g.
/// `&#34;`.
pub struct XmlEscape<T: ?Sized>(pub T);

impl<T: ?Sized + fmt::Display> fmt::Display for XmlEscape<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(XmlEscapeWriter(f), "{}", &self.0)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for XmlEscape<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(XmlEscapeWriter(f), "{:?}", &self.0)
    }
}

struct XmlEscapeWriter<'a, 'b>(&'a mut fmt::Formatter<'b>);

impl fmt::Write for XmlEscapeWriter<'_, '_> {
    fn write_str(&mut self, mut s: &str) -> fmt::Result {
        'outer: loop {
            for (i, c) in s.as_bytes().iter().enumerate() {
                let c = match c {
                    b'"' => "&#34;",
                    b'&' => "&#38;",
                    b'\'' => "&#39;",
                    b'<' => "&#60;",
                    b'>' => "&#62;",
                    _ => continue,
                };
                self.0.write_str(&s[..i])?;
                self.0.write_str(c)?;
                s = &s[i + 1..];
                continue 'outer;
            }
            break self.0.write_str(s);
        }
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.0.write_str(match c {
            '"' => "&#34;",
            '&' => "&#38;",
            '\'' => "&#39;",
            '<' => "&#60;",
            '>' => "&#62;",
            c => return self.0.write_char(c),
        })
    }
}
