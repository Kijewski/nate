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

use std::env::var;
use std::fmt::Write;
use std::fs::OpenOptions;
use std::io::{Read, Write as _};
use std::path::Path;
use std::str::from_utf8;

use memchr::{memchr, memchr2};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::opt;
use nom::error::ErrorKind;
use nom::sequence::{terminated, tuple};
use nom::{error_position, IResult};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, Expr, ExprLit, ExprPath, Lit};

/// Implement [fmt::Display](core::fmt::Display) for a struct or enum.
///
/// Usage:
///
/// ```ignore
/// #[derive(Nate)]
/// #[template(
///     path = "…",
///     output = "…",
/// )]
/// struct Template { /* … */ }
/// ```
///
/// The path is relative to the cargo manifest dir (where you find Cargo.toml) of the calling
/// project.
///
/// The option debug output path is relative to the cargo manifest dir.
/// If supplied the generated code will be written into this file.
/// An existing file fill be replaced!
#[proc_macro_derive(Nate, attributes(template))]
pub fn derive_nate(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let ident = ast.ident.to_string();

    let TemplateAttrs { path, generated } = parse_attributes(ast.attrs);

    let base = var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&base).join(path);
    let output = generated.map(|s| Path::new(&base).join(s));

    let mut content = String::new();
    let generics = &ast.generics;
    write!(
        content,
        "impl {} ::core::fmt::Display for {} ",
        quote!(#generics),
        ident
    )
    .unwrap();
    if !ast.generics.params.is_empty() {
        write!(content, "<").unwrap();
        for arg in ast.generics.params.iter() {
            match arg {
                syn::GenericParam::Type(ty) => {
                    write!(content, " {}, ", ty.ident.to_string()).unwrap();
                }
                syn::GenericParam::Lifetime(def) => {
                    write!(content, " '{}, ", def.lifetime.ident.to_string()).unwrap();
                }
                syn::GenericParam::Const(par) => {
                    write!(content, " {}, ", par.ident.to_string()).unwrap();
                }
            }
        }
        writeln!(content, ">").unwrap();
    }
    if let Some(where_clause) = ast.generics.where_clause {
        writeln!(content, " {} ", quote!(#where_clause)).unwrap();
    }

    writeln!(
        content,
        "{{\nfn fmt(&self, output: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {{"
    )
    .unwrap();
    parse_file(&path, &mut content);
    writeln!(content, "::core::fmt::Result::Ok(())\n}}\n}}").unwrap();

    if let Some(output) = output {
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&output)
            .expect("Could not open output file");
        write!(f, "{}", &content)
            .expect("Could not write to output file after successfully opening it");

        if let Ok(path) = output.canonicalize() {
            if let Some(path) = path.to_str() {
                if let Ok(ts) = format!("::core::include! {{ {:?} }}", path).parse() {
                    return ts;
                }
            }
        }
    }

    content
        .parse()
        .expect("Could not parse generated code as Rust source.")
}

struct TemplateAttrs {
    path: String,
    generated: Option<String>,
}

fn parse_attributes(attrs: Vec<Attribute>) -> TemplateAttrs {
    let mut arguments = None;
    for attr in attrs.into_iter() {
        let Attribute {
            path: syn::Path { segments, .. },
            tokens,
            ..
        } = attr;
        if segments.len() == 1 && segments.first().unwrap().ident == "template" {
            if arguments.is_some() {
                panic!("Duplicated #[template(…)] attribute.");
            }
            arguments = Some(tokens);
        }
    }
    let arguments = arguments.expect("Missing #[template(…)] attribute.");

    let mut path = None;
    let mut generated = None;

    let mut handle_arg = |e: &Expr| {
        let assign = if let Expr::Assign(a) = e {
            a
        } else {
            eprintln!("Expected assignment in #[template(…)] attribute.");
            panic!();
        };

        let left = match &*assign.left {
            Expr::Path(ExprPath {
                path: syn::Path { segments, .. },
                ..
            }) if segments.len() == 1 => segments.first().unwrap().ident.to_string(),
            _ => {
                eprintln!("Expected string literal on RHS in #[template(…)] attribute.");
                panic!()
            }
        };

        let right = if let Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) = &*assign.right
        {
            s.value()
        } else {
            eprintln!("Expected string literal on RHS in #[template(…)] attribute.");
            panic!()
        };

        match left.as_str() {
            "path" => {
                if path.replace(right).is_some() {
                    eprintln!("Duplicated key 'path' in #[template(…)] attribute.");
                    panic!()
                }
            }
            "generated" => {
                if generated.replace(right).is_some() {
                    eprintln!("Duplicated key 'generated' in #[template(…)] attribute.");
                    panic!()
                }
            }
            s => {
                eprintln!("Unexpected key {:?} in #[template(…)] attribute.", s);
                panic!()
            }
        }
    };

    match syn::parse2(arguments)
        .expect("In #[template(…)] you need to use comma separated key=\"value\" pairs.")
    {
        Expr::Paren(p) => handle_arg(&*p.expr),
        Expr::Tuple(g) => {
            for e in g.elems {
                handle_arg(&e);
            }
        }
        _ => {
            eprintln!("Expected arguments in #[template(…)] attribute.");
            panic!();
        }
    }

    TemplateAttrs {
        path: path.expect("Expected key 'path' in #[template(…)] attribute."),
        generated,
    }
}

fn load_file(path: &Path) -> Vec<u8> {
    let mut buf = Vec::new();
    match OpenOptions::new().read(true).open(&path) {
        Ok(mut f) => {
            f.read_to_end(&mut buf)
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
    writeln!(
        output,
        "const _: &'static [u8] = ::core::include_bytes!({:?});\n{{",
        &path
    )
    .unwrap();
    let buf = load_file(path);
    for blocks in parse(path, &buf).into_iter() {
        match blocks {
            ParsedData::Code(blocks) => {
                for code in blocks.into_iter() {
                    writeln!(output, "{}", code).unwrap();
                }
            }
            ParsedData::Data(blocks) => {
                write!(output, "{{\n::core::write!(\noutput,\n\"").unwrap();
                for block in blocks.iter() {
                    match block {
                        DataSection::Data(s) => {
                            let s = format!("{:#?}", s).replace("{", "{{").replace("}", "}}");
                            write!(output, "{}", &s[1..s.len() - 1]).unwrap();
                        }
                        DataSection::Raw(_) | DataSection::Escaped(_) => {
                            write!(output, "{{}}").unwrap();
                        }
                        DataSection::Debug(_) => write!(output, "{{:?}}").unwrap(),
                        DataSection::Verbose(_) => write!(output, "{{:#?}}").unwrap(),
                    }
                }
                writeln!(output, "\",").unwrap();
                for data in blocks.into_iter() {
                    match data {
                        DataSection::Data(_) => {}
                        DataSection::Escaped(s) => {
                            writeln!(output, "::nate::XmlEscape(&({})),", s).unwrap();
                        }
                        DataSection::Raw(s) | DataSection::Debug(s) | DataSection::Verbose(s) => {
                            writeln!(output, "&({}),", s).unwrap();
                        }
                    }
                }
                writeln!(output, ")?;\n}}").unwrap();
            }
        }
    }
    writeln!(output, "}}").unwrap();
}

fn parse(path: &Path, i: &[u8]) -> Vec<ParsedData> {
    WsBlockIter::new(path, i)
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
        .filter(|block| !block.is_empty())
        .fold(Vec::new(), |mut accu, block| {
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
            }
            accu
        })
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
            Block::Data(_) => false,
        }
    }
}

impl<'a> WsBlockIter<'a> {
    fn new(path: &'a Path, i: &'a [u8]) -> Self {
        Self {
            path,
            start: i,
            pos: i,
            next: Some(WsBlock(
                false,
                Block::Code(format!("let _ = include_bytes!({:?});", path)),
                false,
            )),
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
