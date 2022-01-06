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

//! [![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Kijewski/nate/CI)](https://github.com/Kijewski/nate/actions/workflows/ci.yml)
//! [![Crates.io](https://img.shields.io/crates/v/nate-derive)](https://crates.io/crates/nate-derive)
//! [![License](https://img.shields.io/crates/l/nate-derive?color=informational)](/LICENSES)
//!
//! Proc-macros for [NaTE](https://crates.io/crates/nate).
//!
//! This libary implements the `#![derive(Nate)]` annotation.

use std::env::var;
use std::fmt::Write;
use std::fs::OpenOptions;
use std::io::{Read, Write as _};
use std::path::Path;
use std::str::from_utf8;

use darling::FromDeriveInput;
use memchr::{memchr, memchr2};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::opt;
use nom::error::ErrorKind;
use nom::sequence::{terminated, tuple};
use nom::{error_position, IResult};
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

const HEAD: &str = "\
    {\n\
        fn fmt(&self, output: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {\n\
            #[allow(unused_imports)]\n\
            use ::nate::_escape::{\n\
                RawKind as _,\n\
                EscapeKind as _,\n\
            };\n\
";

const TAIL: &str = "\n\
            ::core::fmt::Result::Ok(())\n\
        }\n\
    }\n\
};\n\
";

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
    let ast: DeriveInput = syn::parse(input).unwrap();
    let ident = ast.ident.to_string();

    let opts: TemplateAttrs = match TemplateAttrs::from_derive_input(&ast) {
        Ok(opts) => opts,
        Err(err) => {
            let err = format!("{}", err);
            return Into::into(quote!(
                const _: () = {
                    ::core::compile_error!(#err);
                };
            ));
        }
    };

    let base = var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&base).join(opts.path);
    let output = opts.generated.as_ref().map(|s| Path::new(&base).join(s));

    let mut content = String::new();
    let generics = &ast.generics;
    write!(
        content,
        "const _: () = {{\nimpl {} ::core::fmt::Display for {} ",
        quote!(#generics),
        ident
    )
    .unwrap();
    if !ast.generics.params.is_empty() {
        write!(content, "<").unwrap();
        for arg in ast.generics.params.iter() {
            match arg {
                syn::GenericParam::Type(ty) => {
                    write!(content, " {}, ", ty.ident).unwrap();
                }
                syn::GenericParam::Lifetime(def) => {
                    write!(content, " '{}, ", def.lifetime.ident).unwrap();
                }
                syn::GenericParam::Const(par) => {
                    write!(content, " {}, ", par.ident).unwrap();
                }
            }
        }
        writeln!(content, ">").unwrap();
    }
    if let Some(where_clause) = ast.generics.where_clause {
        writeln!(content, " {} ", quote!(#where_clause)).unwrap();
    }

    write!(content, "{}", HEAD).unwrap();
    parse_file(&path, &mut content);
    write!(content, "{}", TAIL).unwrap();

    if let Some(output) = output {
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&output)
            .expect("Could not open output file");
        write!(f, "{}", &content)
            .expect("Could not write to output file after successfully opening it");
    }

    content
        .parse()
        .expect("Could not parse generated code as Rust source.")
}

#[derive(Debug, Default, FromDeriveInput)]
#[darling(attributes(template))]
struct TemplateAttrs {
    path: String,
    #[darling(default)]
    generated: Option<String>,
}

fn load_file(path: &Path) -> Vec<u8> {
    let mut buf = Vec::new();
    match OpenOptions::new().read(true).open(&path) {
        Ok(mut f) => {
            let _ = f
                .read_to_end(&mut buf)
                .expect("Could not read source file even after successfully opening it.");
        }
        Err(err) => {
            eprintln!("Could not open file={:?}: {:?}", path, err);
            panic!();
        }
    }
    buf
}

fn parse_file(path: &Path, mut output: impl Write) {
    use DataSection::*;

    let buf = load_file(path);
    for (block_index, blocks) in parse(path, &buf).into_iter().enumerate() {
        match blocks {
            ParsedData::Code(blocks) => {
                for code in blocks.into_iter() {
                    writeln!(output, "{}", code).unwrap();
                }
            }
            ParsedData::Data(blocks) => {
                writeln!(output, "{{").unwrap();
                for (data_index, data) in blocks.iter().enumerate() {
                    match data {
                        Data(_) => {}
                        Raw(s) | Escaped(s) | Debug(s) | Verbose(s) => {
                            writeln!(
                                output,
                                "let _nate_arg_{}_{} = &({});",
                                block_index, data_index, s
                            )
                            .unwrap();
                        }
                    }
                }

                for (data_index, data) in blocks.iter().enumerate() {
                    match data {
                        Data(_) | Raw(_) => {}
                        Escaped(_) => {
                            writeln!(
                                output,
                                "let _nate_arg_{block}_{data} = \
                                    (&::nate::_escape::TagWrapper::new(_nate_arg_{block}_{data})).\
                                    wrap(_nate_arg_{block}_{data});",
                                block = block_index,
                                data = data_index,
                            )
                            .unwrap();
                        }
                        Debug(_) | Verbose(_) => {
                            writeln!(
                                output,
                                "let _nate_arg_{block}_{data} = \
                                    ::nate::XmlEscape(_nate_arg_{block}_{data});",
                                block = block_index,
                                data = data_index,
                            )
                            .unwrap();
                        }
                    }
                }

                write!(output, "::core::write!(\noutput,\n\"").unwrap();
                for block in blocks.iter() {
                    match block {
                        Data(s) => {
                            let s = format!("{:#?}", s).replace('{', "{{").replace('}', "}}");
                            write!(output, "{}", &s[1..s.len() - 1]).unwrap();
                        }
                        Raw(_) | Escaped(_) => write!(output, "{{}}").unwrap(),
                        Debug(_) => write!(output, "{{:?}}").unwrap(),
                        Verbose(_) => write!(output, "{{:#?}}").unwrap(),
                    }
                }
                writeln!(output, "\",").unwrap();

                for (data_index, data) in blocks.into_iter().enumerate() {
                    match data {
                        Data(_) => {}
                        Raw(_) | Escaped(_) | Debug(_) | Verbose(_) => {
                            writeln!(output, "_nate_arg_{}_{},", block_index, data_index).unwrap()
                        }
                    }
                }
                writeln!(output, ")?;\n}}").unwrap();
            }
        }
    }
}

fn parse(path: &Path, i: &[u8]) -> Vec<ParsedData> {
    let mut output = Vec::new();
    parse_into(path, i, &mut output);
    output
}

fn parse_into(path: &Path, i: &[u8], accu: &mut Vec<ParsedData>) {
    let it = WsBlockIter::new(path, i)
        .map(|WsBlock(a, b, z)| match b {
            Block::Data(DataSection::Data(s)) => {
                let t = match (a, z) {
                    (true, true) => s.trim(),
                    (true, false) => s.trim_start(),
                    (false, true) => s.trim_end(),
                    (false, false) => &s,
                };
                if s.len() != t.len() {
                    Block::Data(DataSection::Data(t.to_owned()))
                } else {
                    Block::Data(DataSection::Data(s))
                }
            }
            b => b,
        })
        .filter(|block| !block.is_empty());

    let s = format!(
        "{{\nlet _ = (\"start of\", ::core::include_bytes!({:?}));",
        path
    );
    match accu.last_mut() {
        Some(ParsedData::Code(blocks)) => blocks.push(s),
        _ => accu.push(ParsedData::Code(vec![s])),
    }

    for block in it {
        match block {
            Block::Comment => {}
            Block::Code(s) => match accu.last_mut() {
                Some(ParsedData::Code(blocks)) => blocks.push(s),
                _ => accu.push(ParsedData::Code(vec![s])),
            },
            Block::Data(data) => match accu.last_mut() {
                Some(ParsedData::Data(blocks)) => blocks.push(data),
                _ => accu.push(ParsedData::Data(vec![data])),
            },
            Block::Include(include_path) => {
                let include_path = include_path.trim();
                let include_path = match Path::new(include_path).iter().next() {
                    Some(d) if d.eq(".") || d.eq("..") => {
                        path.parent().unwrap_or(path).join(include_path)
                    }
                    _ => Path::new(&var("CARGO_MANIFEST_DIR").unwrap()).join(include_path),
                };
                let buf = load_file(&include_path);
                parse_into(&include_path, &buf, accu);
            }
        }
    }

    let s = format!("let _ = (\"end of\", {:?});\n}}", path);
    match accu.last_mut() {
        Some(ParsedData::Code(blocks)) => blocks.push(s),
        _ => accu.push(ParsedData::Code(vec![s])),
    }
}

#[derive(Debug)]
enum ParsedData {
    Code(Vec<String>),
    Data(Vec<DataSection>),
}

#[derive(Debug, Clone)]
enum DataSection {
    Data(String),
    Raw(String),
    Escaped(String),
    Debug(String),
    Verbose(String),
}

#[derive(Debug, Clone)]
enum Block {
    Data(DataSection),
    Code(String),
    Comment,
    Include(String),
}

struct WsBlock(bool, Block, bool);

struct WsBlockIter<'a> {
    path: &'a Path,
    start: &'a [u8],
    pos: &'a [u8],
    next: Option<WsBlock>,
}

impl Block {
    fn is_empty(&self) -> bool {
        match self {
            Block::Comment => true,
            Block::Code(s) | Block::Data(DataSection::Data(s)) => s.is_empty(),
            Block::Data(_) | Block::Include(_) => false,
        }
    }
}

impl<'a> WsBlockIter<'a> {
    fn new(path: &'a Path, i: &'a [u8]) -> Self {
        Self {
            path,
            start: i,
            pos: i,
            next: Some(WsBlock(false, Block::Comment, false)),
        }
    }
}

fn abort_with_nom_error(err: nom::Err<nom::error::Error<&[u8]>>, start: &[u8], path: &Path) -> ! {
    match err {
        nom::Err::Incomplete(_) => {
            panic!("Impossible");
        }
        nom::Err::Error(err) | nom::Err::Failure(err) => {
            let offset = start.len() - err.input.len();
            let (source_before, source_after) = from_utf8(start).unwrap().split_at(offset);

            let source_after = match source_after.char_indices().enumerate().take(41).last() {
                Some((40, (i, _))) => format!("{:?}...", &source_after[..i]),
                _ => format!("{:?}", source_after),
            };

            let (row, last_line) = source_before.lines().enumerate().last().unwrap();
            let column = last_line.chars().count();

            panic!(
                "Problems parsing template source {:?} at row {}, column {} near:\n{}",
                path,
                row + 1,
                column,
                source_after,
            );
        }
    }
}

impl Iterator for WsBlockIter<'_> {
    type Item = WsBlock;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = if !self.pos.is_empty() {
            match parse_ws_block(self.pos) {
                Ok((j, b)) => {
                    self.pos = j;
                    Some(b)
                }
                Err(err) => {
                    abort_with_nom_error(err, self.start, self.path);
                }
            }
        } else {
            None
        };
        match (cur, &self.next) {
            (Some(WsBlock(next_a, next_b, next_z)), Some(WsBlock(cur_a, cur_b, cur_z))) => {
                let result = WsBlock(*cur_a, cur_b.clone(), cur_z | next_a);
                self.next = Some(WsBlock(next_a || *cur_z, next_b, next_z));
                Some(result)
            }
            (None, _) => self.next.take(),
            (_, None) => panic!("Impossible"),
        }
    }
}

fn parse_ws_block(i: &[u8]) -> IResult<&[u8], WsBlock> {
    alt((
        |i| {
            let (i, (a, b, z)) = parse_block(i, b"{%", b"%}")?;
            Ok((i, WsBlock(a, Block::Code(b), z)))
        },
        |i| {
            let (i, (a, _, z)) = parse_block(i, b"{#", b"#}")?;
            Ok((i, WsBlock(a, Block::Comment, z)))
        },
        |i| {
            let (next_i, (a, b, z)) = parse_block(i, b"{<", b">}")?;
            match b.is_empty() {
                false => Ok((next_i, WsBlock(a, Block::Include(b), z))),
                true => Err(nom::Err::Error(error_position!(i, ErrorKind::NonEmpty))),
            }
        },
        |i| parse_data_section(i, b"{{{{{", b"}}}}}", DataSection::Verbose),
        |i| parse_data_section(i, b"{{{{", b"}}}}", DataSection::Debug),
        |i| parse_data_section(i, b"{{{", b"}}}", DataSection::Raw),
        |i| parse_data_section(i, b"{{", b"}}", DataSection::Escaped),
        parse_data,
    ))(i)
}

fn parse_block<'a>(
    i: &'a [u8],
    start: &'static [u8],
    end: &'static [u8],
) -> IResult<&'a [u8], (bool, String, bool)> {
    let inner = |i: &'a [u8]| -> IResult<&'a [u8], (String, bool)> {
        let mut start = 0;
        while (i.len() - start) >= end.len() {
            if let Some(pos) = memchr2(end[0], b'-', &i[start..]) {
                let (j, end) = opt(terminated(opt(tag(b"-")), tag(end)))(&i[start + pos..])?;
                if let Some(trim) = end {
                    return Ok((
                        j,
                        (
                            from_utf8(&i[..start + pos]).unwrap().trim().to_owned(),
                            trim.is_some(),
                        ),
                    ));
                } else if pos > 0 {
                    start += pos;
                } else {
                    start += 1;
                }
            } else {
                break;
            }
        }
        Ok((b"", (from_utf8(i).unwrap().trim().to_owned(), false)))
    };
    let (i, (_, trim_start, (b, trim_end))) = tuple((tag(start), opt(tag(b"-")), inner))(i)?;
    Ok((i, (trim_start.is_some(), b, trim_end)))
}

fn parse_data_section<'a>(
    i: &'a [u8],
    start: &'static [u8],
    end: &'static [u8],
    kind: impl 'static + Fn(String) -> DataSection,
) -> IResult<&'a [u8], WsBlock> {
    let (j, (trim_start, b, trim_end)) = parse_block(i, start, end)?;
    if !b.is_empty() {
        Ok((j, WsBlock(trim_start, Block::Data(kind(b)), trim_end)))
    } else {
        Err(nom::Err::Error(error_position!(i, ErrorKind::NonEmpty)))
    }
}

fn parse_data<'a>(i: &'a [u8]) -> IResult<&'a [u8], WsBlock> {
    let (i, b): (&'a [u8], _) = if let Some(pos) = memchr(b'{', &i[1..]) {
        let (b, i) = i.split_at(pos + 1);
        (i, b)
    } else {
        (b"", i)
    };
    let b = DataSection::Data(from_utf8(b).unwrap().to_owned());
    Ok((i, WsBlock(false, Block::Data(b), false)))
}
