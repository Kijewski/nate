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

use core::fmt::{self, Write};

/// A wrapper around a [displayable][fmt::Display] type that makes it write out XML escaped.
///
/// All characters are written as is except `"`, `&`, `'`, `<`, and `>` which are printed as e.g.
/// `&#34;`.
pub struct XmlEscape<T: ?Sized + fmt::Display>(pub T);

impl<T: ?Sized + fmt::Display> fmt::Display for XmlEscape<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(XmlEscapeWriter(f), "{}", &self.0)
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
        match dbg!(c) {
            '"' => self.0.write_str("&#34;"),
            '&' => self.0.write_str("&#38;"),
            '\'' => self.0.write_str("&#39;"),
            '<' => self.0.write_str("&#60;"),
            '>' => self.0.write_str("&#62;"),
            c => self.0.write_char(c),
        }
    }
}
