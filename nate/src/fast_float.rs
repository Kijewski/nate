#![cfg(any(feature = "ryu", feature = "ryu-js"))]

#[cfg(feature = "alloc")]
use crate::details::alloc;
use crate::details::ryu::{Buffer, Float};
use crate::details::std::{cell, fmt, num};
use crate::escape::EscapeWrapper;

impl<T: FloatMarker> crate::fast_float::FloatKind for EscapeWrapper<T> {}

/// Types implementing this marker get printed using [ryu](crate::details::ryu)
#[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "ryu", feature = "ryu-js"))))]
pub trait FloatMarker {
    #[doc(hidden)]
    type Escaped: fmt::Display;

    #[doc(hidden)]
    fn escape(&self) -> Self::Escaped;
}

impl<T: FloatMarker> FloatMarker for &T {
    type Escaped = T::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        T::escape(*self)
    }
}

#[doc(hidden)]
pub trait FloatKind {
    #[inline]
    fn wrap<T: FloatMarker>(&self, value: &T) -> <T as FloatMarker>::Escaped {
        value.escape()
    }
}

pub struct FloatEscape<T: Float>(T);

impl<T: Float> fmt::Display for FloatEscape<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(Buffer::new().format(self.0))
    }
}

impl<T: Float> fmt::Debug for FloatEscape<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

impl FloatMarker for f32 {
    type Escaped = FloatEscape<Self>;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        FloatEscape(*self)
    }
}

impl FloatMarker for f64 {
    type Escaped = FloatEscape<Self>;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        FloatEscape(*self)
    }
}

impl<T: FloatMarker> FloatMarker for cell::Ref<'_, T> {
    type Escaped = T::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        let value: &T = &*self;
        value.escape()
    }
}

impl<T: FloatMarker> FloatMarker for cell::RefMut<'_, T> {
    type Escaped = T::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        let value: &T = &*self;
        value.escape()
    }
}

impl<T: FloatMarker> FloatMarker for num::Wrapping<T> {
    type Escaped = T::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        self.0.escape()
    }
}

#[cfg(feature = "alloc")]
const _: () = {
    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "std", feature = "alloc"))))]
    impl<T: FloatMarker + alloc::borrow::ToOwned> FloatMarker for alloc::borrow::Cow<'_, T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = &*self;
            value.escape()
        }
    }

    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "std", feature = "alloc"))))]
    impl<T: FloatMarker> FloatMarker for alloc::boxed::Box<T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = &*self;
            value.escape()
        }
    }

    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "std", feature = "alloc"))))]
    impl<T: FloatMarker> FloatMarker for alloc::rc::Rc<T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = &*self;
            value.escape()
        }
    }

    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "std", feature = "alloc"))))]
    impl<T: FloatMarker> FloatMarker for alloc::sync::Arc<T> {
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
    impl<T: FloatMarker> FloatMarker for sync::MutexGuard<'_, T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = &*self;
            value.escape()
        }
    }

    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "std")))]
    impl<T: FloatMarker> FloatMarker for sync::RwLockReadGuard<'_, T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = &*self;
            value.escape()
        }
    }

    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "std")))]
    impl<T: FloatMarker> FloatMarker for sync::RwLockWriteGuard<'_, T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = &*self;
            value.escape()
        }
    }
};
