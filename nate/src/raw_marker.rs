#![no_implicit_prelude]

use std::prelude::v1::*;

#[cfg(feature = "alloc")]
use super::details::alloc;
use super::details::std;
use super::XmlEscape;

/// Types implementing this marker don't need to be escaped.
pub trait RawMarker {}

impl<T: RawMarker> RawMarker for &T {}
impl<T: RawMarker> RawMarker for &mut T {}

impl<T> RawMarker for XmlEscape<T> {}

impl RawMarker for bool {}
impl RawMarker for f32 {}
impl RawMarker for f64 {}
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

impl RawMarker for std::num::NonZeroI8 {}
impl RawMarker for std::num::NonZeroI16 {}
impl RawMarker for std::num::NonZeroI32 {}
impl RawMarker for std::num::NonZeroI64 {}
impl RawMarker for std::num::NonZeroI128 {}
impl RawMarker for std::num::NonZeroIsize {}
impl RawMarker for std::num::NonZeroU8 {}
impl RawMarker for std::num::NonZeroU16 {}
impl RawMarker for std::num::NonZeroU32 {}
impl RawMarker for std::num::NonZeroU64 {}
impl RawMarker for std::num::NonZeroU128 {}
impl RawMarker for std::num::NonZeroUsize {}

impl<T: RawMarker> RawMarker for std::cell::Ref<'_, T> {}
impl<T: RawMarker> RawMarker for std::cell::RefMut<'_, T> {}
impl<T: RawMarker> RawMarker for std::num::Wrapping<T> {}
impl<T: RawMarker> RawMarker for std::pin::Pin<T> {}

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
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "std")))]
    impl<T: RawMarker> RawMarker for std::sync::MutexGuard<'_, T> {}
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "std")))]
    impl<T: RawMarker> RawMarker for std::sync::RwLockReadGuard<'_, T> {}
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "std")))]
    impl<T: RawMarker> RawMarker for std::sync::RwLockWriteGuard<'_, T> {}
};

#[doc(hidden)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RawTag;

#[doc(hidden)]
#[derive(Debug, Clone, Copy, Default)]
pub struct EscapeTag;

#[doc(hidden)]
impl EscapeTag {
    #[inline]
    pub fn wrap<T>(&self, value: T) -> XmlEscape<T> {
        XmlEscape(value)
    }
}
