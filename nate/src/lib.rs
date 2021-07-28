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

//! ## **N**ot **a** **T**emplate **E**ngine
//!
//! This is not a template engine, but sugar to implicitly call `write!(…)` like in PHP, ASP, and everything you hate.
//! The only difference is that the output gets XML escaped automatically unless opted-out explicitly.
//!
//! E.g.
//!
//! *   templates/greeting.html:  
//!     ```xml
//!     <h1>Hello, {{user}}!</h1>
//!     ```
//! *   src/main.rs:  
//!     ```rs
//!     use nate::Nate;
//!     
//!     #[derive(Nate)]
//!     #[template(path = "templates/greeting.html")]
//!     struct Greetings<'a> {
//!         user: &'a str,
//!     }
//!     
//!     fn main() {
//!         let mut output = String::new();
//!         let tmpl = Greetings { user: "<World>" };
//!         write!(output, "{}", tmpl).unwrap();
//!         println!("{}", output);
//!     }
//!     ```
//! *   Output:
//!     ```html
//!     <h1>Hello, &#60;World&#62;!</h1>
//!     ```
//!
//! No new traits are introduced, instead using `#[derive(Nate)]` works by implementing [fmt::Display].
//! This also makes nesting of NaTE templates possible.
//!
//! A more complex example would be:  
//!
//! *   src/main.rs:  
//!     ```rs
//!     use nate::Nate;
//!
//!     #[derive(Nate)]
//!     #[template(path = "templates/99-bottles.html")]
//!     struct Template {
//!         limit: usize,
//!     }
//!
//!     #[test]
//!     fn ninetynine_bottles_of_beer() {
//!         print!("{}", Template { limit: 99 });
//!     }
//!     ```
//! *   templates/99-bottles.txt:  
//!     ```html
//!     {%-
//!         for i in (1..=self.limit).rev() {
//!             if i == 1 {
//!     -%}
//!     1 bottle of beer on the wall.
//!     1 bottle of beer.
//!     Take one down, pass it around.
//!     {%-
//!             } else {
//!     -%}
//!     {{i}} bottles of beer on the wall.
//!     {{i}} bottles of beer.
//!     Take one down, pass it around.
//!
//!     {%
//!             }
//!         }
//!     -%}
//!     ```
//!
//! Inside of a `{% code block %}` you can write any and all rust code.
//!
//! Values in `{{ value blocks }}` are printed XML escaped.
//!
//! Values in `{{{ raw blocks }}}` are printed verbatim.
//!
//! For values in `{{{{ debug blocks }}}}` their debug message is printed as in `"{:?}"`.
//!
//! For values in `{{{{{ verbose blocks }}}}}` their debug message is printed verbose as in `"{:#?}"`.
//!
//! Using hyphens `-` at the start/end of a block, whitespaces before/after the block are trimmed.

#![forbid(unsafe_code)]
#![no_std]

pub use nate_common::XmlEscape;
pub use nate_derive::Nate;

#[cfg(doc)]
use core::fmt;
