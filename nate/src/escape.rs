#[cfg(feature = "alloc")]
use crate::details::alloc;
use crate::details::std::fmt::{self, Write as _};
use crate::details::std::marker::PhantomData;
use crate::details::std::prelude::v1::*;
use crate::details::std::{cell, num, write};

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
pub trait RawKind {
    #[inline]
    fn wrap<'a, T: RawMarker>(&self, value: &'a T) -> &'a T {
        value
    }
}

#[doc(hidden)]
pub trait EscapeKind {
    #[inline]
    fn wrap<'a, T>(&self, value: &'a T) -> XmlEscape<&'a T> {
        XmlEscape(value)
    }
}

/// A wrapper around a [displayable][fmt::Display] type that makes it write out XML escaped.
///
/// All characters are written as is except `"`, `&`, `'`, `<`, and `>` which are printed as e.g.
/// `&#34;`.
pub struct XmlEscape<T: ?Sized>(pub T);

const _: () = {
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
};

impl<T: RawMarker> RawKind for EscapeWrapper<T> {}

impl<T> EscapeKind for &EscapeWrapper<T> {}

/// Types implementing this marker don't need to be escaped
pub trait RawMarker {}

impl<T: RawMarker> RawMarker for &T {}
impl<T> RawMarker for XmlEscape<T> {}

impl RawMarker for bool {}

impl<T: RawMarker> RawMarker for cell::Ref<'_, T> {}
impl<T: RawMarker> RawMarker for cell::RefMut<'_, T> {}
impl<T: RawMarker> RawMarker for num::Wrapping<T> {}

#[cfg(feature = "alloc")]
const _: () = {
    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "std", feature = "alloc"))))]
    impl<T: RawMarker + alloc::borrow::ToOwned> RawMarker for alloc::borrow::Cow<'_, T> {}
    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "std", feature = "alloc"))))]
    impl<T: RawMarker> RawMarker for alloc::boxed::Box<T> {}
    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "std", feature = "alloc"))))]
    impl<T: RawMarker> RawMarker for alloc::rc::Rc<T> {}
    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "std", feature = "alloc"))))]
    impl<T: RawMarker> RawMarker for alloc::sync::Arc<T> {}
};

#[cfg(feature = "std")]
const _: () = {
    use super::details::std::sync;

    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "std")))]
    impl<T: RawMarker> RawMarker for sync::MutexGuard<'_, T> {}
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "std")))]
    impl<T: RawMarker> RawMarker for sync::RwLockReadGuard<'_, T> {}
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "std")))]
    impl<T: RawMarker> RawMarker for sync::RwLockWriteGuard<'_, T> {}
};

#[cfg(not(feature = "itoa"))]
const _: () = {
    impl RawMarker for i128 {}
    impl RawMarker for i16 {}
    impl RawMarker for i32 {}
    impl RawMarker for i64 {}
    impl RawMarker for i8 {}
    impl RawMarker for isize {}
    impl RawMarker for u128 {}
    impl RawMarker for u16 {}
    impl RawMarker for u32 {}
    impl RawMarker for u64 {}
    impl RawMarker for u8 {}
    impl RawMarker for usize {}

    impl RawMarker for num::NonZeroI8 {}
    impl RawMarker for num::NonZeroI16 {}
    impl RawMarker for num::NonZeroI32 {}
    impl RawMarker for num::NonZeroI64 {}
    impl RawMarker for num::NonZeroI128 {}
    impl RawMarker for num::NonZeroIsize {}
    impl RawMarker for num::NonZeroU8 {}
    impl RawMarker for num::NonZeroU16 {}
    impl RawMarker for num::NonZeroU32 {}
    impl RawMarker for num::NonZeroU64 {}
    impl RawMarker for num::NonZeroU128 {}
    impl RawMarker for num::NonZeroUsize {}
};

#[cfg(not(any(feature = "ryu", feature = "ryu-js")))]
const _: () = {
    impl RawMarker for f32 {}
    impl RawMarker for f64 {}
};
