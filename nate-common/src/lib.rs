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

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![no_std]

//! Helper library for [NaTE](https://crates.io/crates/nate).
//!
//! This libary code used during the runtime of the generated code.

use core::fmt::{self, Write};

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

/// Types implething this marker don't need to be escaped.
pub trait RawMarker {}

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

impl<T: RawMarker> RawMarker for &T {}

#[doc(hidden)]
pub struct RawTag;

#[doc(hidden)]
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
    use core::marker::PhantomData;

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
