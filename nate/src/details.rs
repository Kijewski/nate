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

#[doc(hidden)]
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
