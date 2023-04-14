#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use core::{cell, fmt, num};

use crate::details::EscapeWrapper;

impl<T: RawMarker> RawKind for &&&EscapeWrapper<T> {}

/// Types implementing this marker are not escaped, but printed verbatim
pub trait RawMarker: fmt::Display {
    #[doc(hidden)]
    type Escaped: fmt::Display;

    #[doc(hidden)]
    fn escape(&self) -> Self::Escaped;
}

impl<T: RawMarker> RawMarker for &T {
    type Escaped = T::Escaped;

    fn escape(&self) -> Self::Escaped {
        T::escape(*self)
    }
}

#[doc(hidden)]
pub trait RawKind {
    #[inline]
    fn wrap<T: RawMarker>(&self, value: &T) -> <T as RawMarker>::Escaped {
        value.escape()
    }
}

impl RawMarker for bool {
    type Escaped = bool;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        *self
    }
}

impl<T: RawMarker> RawMarker for cell::Ref<'_, T> {
    type Escaped = T::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        let value: &T = self;
        value.escape()
    }
}

impl<T: RawMarker> RawMarker for cell::RefMut<'_, T> {
    type Escaped = T::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        let value: &T = self;
        value.escape()
    }
}

impl<T: RawMarker> RawMarker for num::Wrapping<T> {
    type Escaped = T::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        self.0.escape()
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
const _: () = {
    impl<T> RawMarker for alloc::borrow::Cow<'_, T>
    where
        T: RawMarker + alloc::borrow::ToOwned,
        <T as alloc::borrow::ToOwned>::Owned: fmt::Display,
    {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = self;
            value.escape()
        }
    }

    impl<T: RawMarker> RawMarker for alloc::boxed::Box<T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = self;
            value.escape()
        }
    }

    impl<T: RawMarker> RawMarker for alloc::rc::Rc<T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = self;
            value.escape()
        }
    }

    impl<T: RawMarker> RawMarker for alloc::sync::Arc<T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = self;
            value.escape()
        }
    }
};

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
const _: () = {
    use std::sync;

    impl<T: RawMarker> RawMarker for sync::MutexGuard<'_, T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = self;
            value.escape()
        }
    }

    impl<T: RawMarker> RawMarker for sync::RwLockReadGuard<'_, T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = self;
            value.escape()
        }
    }

    impl<T: RawMarker> RawMarker for sync::RwLockWriteGuard<'_, T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = self;
            value.escape()
        }
    }
};
