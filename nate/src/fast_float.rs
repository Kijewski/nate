#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use core::marker::Copy;
use core::{cell, fmt, num};

use ryu::{Buffer, Float};

use crate::details::EscapeWrapper;

impl<T: FloatMarker> FloatKind for &&EscapeWrapper<T> {}

/// Types implementing this marker get printed using [`ryu`](ryu)
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

pub struct FloatEscape<T: Float + Copy>(T);

impl<T: Float + Copy> fmt::Display for FloatEscape<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(Buffer::new().format(self.0))
    }
}

impl<T: Float + Copy> fmt::Debug for FloatEscape<T> {
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
        let value: &T = self;
        value.escape()
    }
}

impl<T: FloatMarker> FloatMarker for cell::RefMut<'_, T> {
    type Escaped = T::Escaped;

    #[inline]
    fn escape(&self) -> Self::Escaped {
        let value: &T = self;
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
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
const _: () = {
    impl<T: FloatMarker + alloc::borrow::ToOwned> FloatMarker for alloc::borrow::Cow<'_, T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = self;
            value.escape()
        }
    }

    impl<T: FloatMarker> FloatMarker for alloc::boxed::Box<T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = self;
            value.escape()
        }
    }

    impl<T: FloatMarker> FloatMarker for alloc::rc::Rc<T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = self;
            value.escape()
        }
    }

    impl<T: FloatMarker> FloatMarker for alloc::sync::Arc<T> {
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

    impl<T: FloatMarker> FloatMarker for sync::MutexGuard<'_, T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = self;
            value.escape()
        }
    }

    impl<T: FloatMarker> FloatMarker for sync::RwLockReadGuard<'_, T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = self;
            value.escape()
        }
    }

    impl<T: FloatMarker> FloatMarker for sync::RwLockWriteGuard<'_, T> {
        type Escaped = T::Escaped;

        #[inline]
        fn escape(&self) -> Self::Escaped {
            let value: &T = self;
            value.escape()
        }
    }
};
