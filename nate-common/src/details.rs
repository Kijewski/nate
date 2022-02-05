#![no_implicit_prelude]

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
use std::fmt::{self, Arguments};
use std::marker::PhantomData;
use std::prelude::v1::*;

#[cfg(feature = "alloc")]
pub(crate) struct WriteIo<W: alloc::io::Write>(pub(crate) W);

pub(crate) struct WriteFmt<W: fmt::Write>(pub(crate) W);

#[cfg(feature = "alloc")]
impl<W: alloc::io::Write> super::WriteAny for WriteIo<W> {
    #[inline]
    fn write_fmt(&mut self, fmt: Arguments<'_>) -> fmt::Result {
        match <W as alloc::io::Write>::write_fmt(&mut self.0, fmt) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }
}

#[cfg(feature = "alloc")]
impl<W: fmt::Write> super::WriteAny for WriteFmt<W> {
    #[inline]
    fn write_fmt(&mut self, fmt: Arguments<'_>) -> fmt::Result {
        <W as fmt::Write>::write_fmt(&mut self.0, fmt)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TagWrapper<E>(PhantomData<fn() -> *const E>);

impl<E> TagWrapper<E> {
    pub fn new(_: &E) -> Self {
        Self(PhantomData)
    }
}

pub trait RawKind {
    fn wrap<'a, T: super::RawMarker>(&self, value: &'a T) -> &'a T {
        value
    }
}

pub trait EscapeKind {
    fn wrap<'a, T>(&self, value: &'a T) -> super::XmlEscape<&'a T> {
        super::XmlEscape(value)
    }
}

impl<T: super::RawMarker> RawKind for TagWrapper<T> {}

impl<T> EscapeKind for &TagWrapper<T> {}
