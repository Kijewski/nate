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

    const MIN_CHAR: u8 = b'"';
    const MAX_CHAR: u8 = b'>';
    const TABLE: [Option<&&str>; (MAX_CHAR - MIN_CHAR + 1) as usize] = {
        let mut table = [None; (MAX_CHAR - MIN_CHAR + 1) as usize];
        table[(b'"' - MIN_CHAR) as usize] = Some(&"&#34;");
        table[(b'&' - MIN_CHAR) as usize] = Some(&"&#38;");
        table[(b'\'' - MIN_CHAR) as usize] = Some(&"&#39;");
        table[(b'<' - MIN_CHAR) as usize] = Some(&"&#60;");
        table[(b'>' - MIN_CHAR) as usize] = Some(&"&#62;");
        table
    };

    impl fmt::Write for XmlEscapeWriter<'_, '_> {
        fn write_str(&mut self, string: &str) -> fmt::Result {
            let mut last = 0;
            for (index, byte) in string.bytes().enumerate() {
                let escaped = match byte {
                    MIN_CHAR..=MAX_CHAR => TABLE[(byte - MIN_CHAR) as usize],
                    _ => None,
                };
                if let Some(escaped) = escaped {
                    self.0.write_str(&string[last..index])?;
                    self.0.write_str(escaped)?;
                    last = index + 1;
                }
            }
            self.0.write_str(&string[last..])
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
    #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
    impl<T: RawMarker + alloc::borrow::ToOwned> RawMarker for alloc::borrow::Cow<'_, T> {}
    #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
    impl<T: RawMarker> RawMarker for alloc::boxed::Box<T> {}
    #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
    impl<T: RawMarker> RawMarker for alloc::rc::Rc<T> {}
    #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
    impl<T: RawMarker> RawMarker for alloc::sync::Arc<T> {}
};

#[cfg(feature = "std")]
const _: () = {
    use super::details::std::sync;

    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    impl<T: RawMarker> RawMarker for sync::MutexGuard<'_, T> {}
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    impl<T: RawMarker> RawMarker for sync::RwLockReadGuard<'_, T> {}
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
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
