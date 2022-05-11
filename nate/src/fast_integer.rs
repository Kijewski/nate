#![cfg(feature = "itoa")]

#[cfg(feature = "alloc")]
use crate::details::alloc;
use crate::details::itoa::{Buffer, Integer};
use crate::details::std::{cell, fmt, num};
use crate::escape::EscapeWrapper;

impl<T: IntMarker> crate::fast_integer::IntKind for EscapeWrapper<T> {}

/// Types implementing this marker get printed using [itoa](crate::details::itoa)
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "itoa")))]
pub trait IntMarker {
    #[doc(hidden)]
    type Escaped: fmt::Display;

    #[doc(hidden)]
    fn escape(&self) -> Self::Escaped;
}

impl<T: IntMarker> IntMarker for &T {
    type Escaped = T::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        T::escape(*self)
    }
}

#[doc(hidden)]
pub trait IntKind {
    #[inline]
    fn wrap<T: IntMarker>(&self, value: &T) -> <T as IntMarker>::Escaped {
        value.escape()
    }
}

pub struct ItoaEscape<T: Integer>(T);

impl<T: Integer> fmt::Display for ItoaEscape<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(Buffer::new().format(self.0))
    }
}

impl<T: Integer> fmt::Debug for ItoaEscape<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

impl IntMarker for i128 {
    type Escaped = ItoaEscape<Self>;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        ItoaEscape(*self)
    }
}

impl IntMarker for i16 {
    type Escaped = ItoaEscape<Self>;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        ItoaEscape(*self)
    }
}

impl IntMarker for i32 {
    type Escaped = ItoaEscape<Self>;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        ItoaEscape(*self)
    }
}

impl IntMarker for i64 {
    type Escaped = ItoaEscape<Self>;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        ItoaEscape(*self)
    }
}

impl IntMarker for i8 {
    type Escaped = ItoaEscape<Self>;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        ItoaEscape(*self)
    }
}

impl IntMarker for isize {
    type Escaped = ItoaEscape<Self>;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        ItoaEscape(*self)
    }
}

impl IntMarker for u128 {
    type Escaped = ItoaEscape<Self>;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        ItoaEscape(*self)
    }
}

impl IntMarker for u16 {
    type Escaped = ItoaEscape<Self>;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        ItoaEscape(*self)
    }
}

impl IntMarker for u32 {
    type Escaped = ItoaEscape<Self>;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        ItoaEscape(*self)
    }
}

impl IntMarker for u64 {
    type Escaped = ItoaEscape<Self>;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        ItoaEscape(*self)
    }
}

impl IntMarker for u8 {
    type Escaped = ItoaEscape<Self>;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        ItoaEscape(*self)
    }
}

impl IntMarker for usize {
    type Escaped = ItoaEscape<Self>;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        ItoaEscape(*self)
    }
}

impl IntMarker for num::NonZeroI8 {
    type Escaped = <i8 as IntMarker>::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        self.get().escape()
    }
}

impl IntMarker for num::NonZeroI16 {
    type Escaped = <i16 as IntMarker>::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        self.get().escape()
    }
}

impl IntMarker for num::NonZeroI32 {
    type Escaped = <i32 as IntMarker>::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        self.get().escape()
    }
}

impl IntMarker for num::NonZeroI64 {
    type Escaped = <i64 as IntMarker>::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        self.get().escape()
    }
}

impl IntMarker for num::NonZeroI128 {
    type Escaped = <i128 as IntMarker>::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        self.get().escape()
    }
}

impl IntMarker for num::NonZeroIsize {
    type Escaped = <isize as IntMarker>::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        self.get().escape()
    }
}

impl IntMarker for num::NonZeroU8 {
    type Escaped = <u8 as IntMarker>::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        self.get().escape()
    }
}

impl IntMarker for num::NonZeroU16 {
    type Escaped = <u16 as IntMarker>::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        self.get().escape()
    }
}

impl IntMarker for num::NonZeroU32 {
    type Escaped = <u32 as IntMarker>::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        self.get().escape()
    }
}

impl IntMarker for num::NonZeroU64 {
    type Escaped = <u64 as IntMarker>::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        self.get().escape()
    }
}

impl IntMarker for num::NonZeroU128 {
    type Escaped = <u128 as IntMarker>::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        self.get().escape()
    }
}

impl IntMarker for num::NonZeroUsize {
    type Escaped = <usize as IntMarker>::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        self.get().escape()
    }
}

impl<T: IntMarker> IntMarker for cell::Ref<'_, T> {
    type Escaped = T::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        let value: &T = &*self;
        value.escape()
    }
}

impl<T: IntMarker> IntMarker for cell::RefMut<'_, T> {
    type Escaped = T::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        let value: &T = &*self;
        value.escape()
    }
}

impl<T: IntMarker> IntMarker for num::Wrapping<T> {
    type Escaped = T::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        self.0.escape()
    }
}

#[cfg(feature = "alloc")]
const _: () = {
    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "std", feature = "alloc"))))]
    impl<T: IntMarker + alloc::borrow::ToOwned> IntMarker for alloc::borrow::Cow<'_, T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = &*self;
            value.escape()
        }
    }

    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "std", feature = "alloc"))))]
    impl<T: IntMarker> IntMarker for alloc::boxed::Box<T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = &*self;
            value.escape()
        }
    }

    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "std", feature = "alloc"))))]
    impl<T: IntMarker> IntMarker for alloc::rc::Rc<T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = &*self;
            value.escape()
        }
    }

    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "std", feature = "alloc"))))]
    impl<T: IntMarker> IntMarker for alloc::sync::Arc<T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = &*self;
            value.escape()
        }
    }
};

#[cfg(feature = "std")]
const _: () = {
    use crate::details::std::sync;

    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "std")))]
    impl<T: IntMarker> IntMarker for sync::MutexGuard<'_, T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = &*self;
            value.escape()
        }
    }

    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "std")))]
    impl<T: IntMarker> IntMarker for sync::RwLockReadGuard<'_, T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = &*self;
            value.escape()
        }
    }

    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "std")))]
    impl<T: IntMarker> IntMarker for sync::RwLockWriteGuard<'_, T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = &*self;
            value.escape()
        }
    }
};
