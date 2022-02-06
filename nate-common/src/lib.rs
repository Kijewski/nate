// Copyright (c) 2021-2022 René Kijewski <crates.io@k6i.de>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_std]
#![cfg_attr(feature = "docsrs", feature(doc_cfg))]
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
#![no_implicit_prelude]

//! [![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Kijewski/nate/CI)](https://github.com/Kijewski/nate/actions/workflows/ci.yml)
//! [![Crates.io](https://img.shields.io/crates/v/nate-common)](https://crates.io/crates/nate-common)
//! [![License](https://img.shields.io/crates/l/nate-common?color=informational)](/LICENSES)
//!
//! Helper library for [NaTE](https://crates.io/crates/nate).
//!
//! This libary is used during the runtime of the generated code.
//!
//! ## Feature flags
#![cfg_attr(feature = "docsrs", doc = ::document_features::document_features!())]

#[doc(hidden)]
pub mod details;
mod raw_marker;

use details::alloc;
use details::std::fmt::{self, Arguments, Write as _};
use details::std::prelude::v1::*;
use details::std::write;
pub use raw_marker::{EscapeTag, RawMarker, RawTag};

#[doc(hidden)]
pub trait WriteAny {
    fn write_fmt(&mut self, fmt: Arguments<'_>) -> fmt::Result;
}

/// Optimized trait methods to render a NaTE template
///
/// Every NaTE template implements this trait.
pub trait RenderInto {
    #[doc(hidden)]
    fn render_into(&self, output: impl WriteAny) -> fmt::Result;

    /// Render the output into an fmt::Write object
    #[inline]
    fn render_fmt(&self, output: impl fmt::Write) -> fmt::Result {
        self.render_into(details::WriteFmt(output))
    }

    /// Render the output into an io::Write object
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "alloc", feature = "std"))))]
    #[inline]
    fn render_io(&self, output: impl alloc::io::Write) -> fmt::Result {
        self.render_into(details::WriteIo(output))
    }

    /// Render the output into a new string
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "alloc", feature = "std"))))]
    fn render_string(&self) -> Result<alloc::string::String, fmt::Error> {
        let mut result = String::new();
        self.render_fmt(&mut result)?;
        Ok(result)
    }

    /// Render the output into a new vector
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "docsrs", doc(cfg(any(feature = "alloc", feature = "std"))))]
    fn render_bytes(&self) -> Result<alloc::vec::Vec<u8>, fmt::Error> {
        let mut result = Vec::new();
        self.render_io(&mut result)?;
        Ok(result)
    }
}

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
