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
//
// --- LLVM Exceptions to the Apache 2.0 License ----
//
// As an exception, if, as a result of your compiling your source code, portions
// of this Software are embedded into an Object form of such source code, you
// may redistribute such embedded portions in such Object form without complying
// with the conditions of Sections 4(a), 4(b) and 4(d) of the License.
//
// In addition, if you combine or link compiled forms of this Software with
// software that is licensed under the GPLv2 ("Combined Software") and if a
// court of competent jurisdiction determines that the patent provision (Section
// 3), the indemnity provision (Section 9) or other Section of the License
// conflicts with the conditions of the GPLv2, you may retroactively and
// prospectively choose to deem waived or otherwise exclude such Section(s) of
// the License, but only in their entirety and only with respect to the Combined
// Software.

#![forbid(unsafe_code)]
#![allow(unused_attributes)]
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
#![allow(clippy::many_single_char_names)]

//! ## NaTE — Not a Template Engine
//!
//! [![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Kijewski/nate/CI?logo=github)](https://github.com/Kijewski/nate/actions/workflows/ci.yml)
//! [![Crates.io](https://img.shields.io/crates/v/nate-derive?logo=rust)](https://crates.io/crates/nate)
//! ![Minimum supported Rust version](https://img.shields.io/badge/rustc-1.53+-important?logo=rust "Minimum Supported Rust Version")
//! [![License](https://img.shields.io/badge/license-Apache--2.0%20WITH%20LLVM--exception-informational?logo=apache)](https://github.com/Kijewski/nate/blob/v0.2.0/LICENSE "Apache-2.0 WITH LLVM-exception")
//!
//! Proc-macros for [NaTE](https://crates.io/crates/nate).
//!
//! This libary implements the `#![derive(Nate)]` annotation.

mod compile_error;
mod generate;
mod nate_span;
mod parse;
mod strip;

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;

use crate::generate::generate;
use crate::strip::Strip;

/// Implement [fmt::Display](core::fmt::Display) for a struct or enum.
///
/// Usage:
///
/// ```ignore
/// #[derive(Nate)]
/// #[template(
///     path = "…",
///     generated = "…",
/// )]
/// struct Template { /* … */ }
/// ```
///
/// The path is relative to the cargo manifest dir (where you find Cargo.toml) of the calling
/// project.
///
/// The optional debug output path `generated` is relative to the cargo manifest dir.
/// If supplied the generated code will be written into this file.
/// An existing file fill be replaced!
#[proc_macro_derive(Nate, attributes(template))]
pub fn derive_nate(input: TokenStream) -> TokenStream {
    let err = match generate(input) {
        Ok(ts) => return ts,
        Err(err) => err,
    };

    let err = format!("{}", err);
    Into::into(quote!(
        const _: () = {
            ::nate::details::std::compile_error!(#err);
        };
    ))
}

#[derive(Debug, Default, FromDeriveInput)]
#[darling(attributes(template))]
struct TemplateAttrs {
    path: String,
    #[darling(default)]
    generated: Option<String>,
    #[darling(default)]
    #[allow(unused)] // TODO
    strip: Strip,
}
