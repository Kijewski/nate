// Copyright (c) 2021 René Kijewski <rene.[SURNAME]@fu-berlin.de>
// All rights reserved.
//
// This software and the accompanying materials are made available under
// the terms of the ISC License which is available in the project root as LICENSE-ISC, AND/OR
// the terms of the MIT License which is available at in the project root as LICENSE-MIT, AND/OR
// the terms of the Apache License, Version 2.0 which is available in the project root as LICENSE-APACHE.
//
// You have to accept AT LEAST one of the aforementioned licenses to use, copy, modify, and/or distribute this software.
// At your will you may redistribute the software under the terms of only one, two, or all three of the aforementioned licenses.

#![no_std]
#![forbid(unsafe_code)]
#![warn(absolute_paths_not_starting_with_crate)]
#![warn(elided_lifetimes_in_paths)]
#![warn(explicit_outlives_requirements)]
#![warn(meta_variable_misuse)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(noop_method_call)]
#![warn(single_use_lifetimes)]
#![warn(trivial_casts)]
#![warn(unreachable_pub)]
#![warn(unused_crate_dependencies)]
#![warn(unused_extern_crates)]
#![warn(unused_lifetimes)]
#![warn(unused_results)]

//! Helper library for [NaTE](https://crates.io/crates/nate).
//!
//! This libary code used during the runtime of the generated code.

#[cfg(feature = "std")]
extern crate std;
#[cfg(not(feature = "std"))]
use core as std;

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;
#[cfg(feature = "std")]
use std as alloc;

use std::fmt::{self, Write};

/// A wrapper around a [displayable][fmt::Display] type that makes it write out XML escaped.
///
/// All characters are written as is except `"`, `&`, `'`, `<`, and `>` which are printed as e.g.
/// `&#34;`.
pub struct XmlEscape<T: ?Sized>(pub T);

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
        loop {
            let mut done = true;
            for (i, c) in s.as_bytes().iter().enumerate() {
                let c = match c {
                    b'"' => "&#34;",
                    b'&' => "&#38;",
                    b'\'' => "&#39;",
                    b'<' => "&#60;",
                    b'>' => "&#62;",
                    _ => continue,
                };
                write!(self.0, "{}{}", &s[..i], c)?;
                s = &s[i + 1..];
                done = false;
                break;
            }
            if done {
                if !s.is_empty() {
                    self.0.write_str(s)?;
                }
                break Ok(());
            }
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

/// Types implementing this marker don't need to be escaped.
pub trait RawMarker {}

impl<T: RawMarker> RawMarker for &T {}

impl<T> RawMarker for XmlEscape<T> {}

impl RawMarker for std::primitive::bool {}
impl RawMarker for std::primitive::f32 {}
impl RawMarker for std::primitive::f64 {}
impl RawMarker for std::primitive::i128 {}
impl RawMarker for std::primitive::i16 {}
impl RawMarker for std::primitive::i32 {}
impl RawMarker for std::primitive::i64 {}
impl RawMarker for std::primitive::i8 {}
impl RawMarker for std::primitive::isize {}
impl RawMarker for std::primitive::u128 {}
impl RawMarker for std::primitive::u16 {}
impl RawMarker for std::primitive::u32 {}
impl RawMarker for std::primitive::u64 {}
impl RawMarker for std::primitive::u8 {}
impl RawMarker for std::primitive::usize {}

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
impl<T: RawMarker + alloc::borrow::ToOwned> RawMarker for alloc::borrow::Cow<'_, T> {}
#[cfg(feature = "alloc")]
impl<T: RawMarker> RawMarker for alloc::boxed::Box<T> {}
#[cfg(feature = "alloc")]
impl<T: RawMarker> RawMarker for alloc::rc::Rc<T> {}
#[cfg(feature = "alloc")]
impl<T: RawMarker> RawMarker for alloc::sync::Arc<T> {}

#[cfg(feature = "std")]
impl<T: RawMarker> RawMarker for std::sync::MutexGuard<'_, T> {}
#[cfg(feature = "std")]
impl<T: RawMarker> RawMarker for std::sync::RwLockReadGuard<'_, T> {}
#[cfg(feature = "std")]
impl<T: RawMarker> RawMarker for std::sync::RwLockWriteGuard<'_, T> {}

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

#[doc(hidden)]
pub mod _escape {
    use crate::std;
    use std::marker::PhantomData;

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
}
