#[cfg(all(feature = "alloc", not(feature = "std")))]
pub extern crate alloc;

#[cfg(not(feature = "alloc"))]
pub mod alloc {}

#[cfg(feature = "std")]
pub extern crate std;

#[cfg(not(feature = "std"))]
pub extern crate core as std;

#[cfg(feature = "std")]
pub use std as alloc;

pub mod itoa {
    #[cfg(feature = "itoa")]
    pub use ::itoa_::{Buffer, Integer};
}

pub mod ryu {
    #[cfg(all(feature = "ryu", not(feature = "ryu-js")))]
    pub use ::ryu_::{Buffer, Float};
    #[cfg(feature = "ryu-js")]
    pub use ::ryu_js_::{Buffer, Float};
}

pub use crate::escape::{EscapeKind, EscapeWrapper, RawKind, XmlEscape};
#[cfg(feature = "ryu")]
pub use crate::fast_float::FloatKind;
#[cfg(feature = "itoa")]
pub use crate::fast_integer::IntKind;

#[doc(hidden)]
#[cfg(not(any(feature = "ryu", feature = "ryu-js")))]
pub trait FloatKind {}

#[doc(hidden)]
#[cfg(not(feature = "itoa"))]
pub trait IntKind {}

#[doc(hidden)]
pub trait WriteAny {
    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::fmt::Result;
    fn write_str(&mut self, s: &str) -> std::fmt::Result;
}

#[cfg(feature = "alloc")]
pub(crate) struct WriteIo<W: alloc::io::Write>(pub(crate) W);

pub(crate) struct WriteFmt<W: std::fmt::Write>(pub(crate) W);

#[cfg(feature = "alloc")]
impl<W: alloc::io::Write> WriteAny for WriteIo<W> {
    #[inline]
    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::fmt::Result {
        match <W as alloc::io::Write>::write_fmt(&mut self.0, fmt) {
            std::result::Result::Ok(_) => std::result::Result::Ok(()),
            std::result::Result::Err(_) => std::result::Result::Err(std::fmt::Error),
        }
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.write_fmt(format_args!("{}", s))
    }
}

impl<W: std::fmt::Write> WriteAny for WriteFmt<W> {
    #[inline]
    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::fmt::Result {
        <W as std::fmt::Write>::write_fmt(&mut self.0, fmt)
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        <W as std::fmt::Write>::write_str(&mut self.0, s)
    }
}

/// Optimized trait methods to render a NaTE template
///
/// Every NaTE template implements this trait.
pub trait RenderInto {
    #[doc(hidden)]
    fn render_into(&self, output: impl WriteAny) -> std::fmt::Result;

    /// Render the output into an fmt::Write object
    #[inline]
    fn render_fmt(&self, output: impl std::fmt::Write) -> std::fmt::Result {
        self.render_into(WriteFmt(output))
    }

    /// Render the output into an io::Write object
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "alloc", feature = "std"))))]
    #[inline]
    fn render_io(&self, output: impl alloc::io::Write) -> std::fmt::Result {
        self.render_into(WriteIo(output))
    }

    /// Render the output into a new string
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "alloc", feature = "std"))))]
    fn render_string(&self) -> std::result::Result<alloc::string::String, std::fmt::Error> {
        let mut result = alloc::string::String::new();
        self.render_fmt(&mut result)?;
        std::result::Result::Ok(result)
    }

    /// Render the output into a new vector
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "alloc", feature = "std"))))]
    fn render_bytes(&self) -> std::result::Result<alloc::vec::Vec<u8>, std::fmt::Error> {
        let mut result = alloc::vec::Vec::new();
        self.render_io(&mut result)?;
        std::result::Result::Ok(result)
    }
}
