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
use std::io::Read;
use std::path::Path;
use std::str::from_utf8;

use memchr::{memchr, memchr2};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{cut, eof, not, opt};
use nom::sequence::{preceded, terminated, tuple};
use nom::IResult;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, Expr, ExprLit, ExprPath, Lit};

#[cfg(doc)]
use std::fmt;

/// Implement [fmt::Display](fmt::Display) for a struct or enum.
///
/// Usage:
///
/// ```rs
/// #[derive(Nate)]
/// #[template(path = "…")]
/// struct …
/// ```
///
/// The path is relative to the cargo mafinest dir (where you find Cargo.toml) of the calling
/// project.
#[proc_macro_derive(Nate, attributes(template))]
pub fn derive_nate(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let ident = ast.ident.to_string();

    let TemplateAttr::Path(path) = parse_attributes(ast.attrs);

    let base = var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&base).join(&path);
    let f = OpenOptions::new().read(true).open(&path);
    let f = match f {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Could not open file={:?}: {:?}", path, err);
            panic!();
        }
    };

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
        r#"{{
fn fmt(&self, output: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {{
const _: &'static [u8] = ::core::include_bytes!({:?});"#,
        path
    )
    .unwrap();
    parse_file(f, &mut content);
    writeln!(content, "::core::fmt::Result::Ok(())\n}}\n}}").unwrap();

    content.parse().unwrap()
}

enum TemplateAttr {
    Path(String),
}

fn parse_attributes(attrs: Vec<Attribute>) -> TemplateAttr {
    let mut arguments = None;
    for Attribute {
        path: syn::Path { segments, .. },
        tokens,
        ..
    } in attrs.into_iter()
    {
        if segments.len() == 1 && segments.first().unwrap().ident == "template" {
            if arguments.is_some() {
                panic!("Duplicated #[template(…)] attribute.");
            }
            arguments = Some(tokens);
        }
    }
    let arguments = arguments.expect("Missing #[template(…)] attribute.");

    let mut path = None;

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
            s => {
                eprintln!("Unexpected key {:?} in #[template(…)] attribute.", s);
                panic!()
            }
        }
    };

    match syn::parse2(arguments).unwrap() {
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

    match path {
        Some(path) => TemplateAttr::Path(path),
        None => {
            eprintln!("Expected key 'path' in #[template(…)] attribute.");
            panic!();
        }
    }
}

fn parse_file(mut input: impl Read, mut output: impl Write) {
    let mut buf = Vec::new();
    input.read_to_end(&mut buf).unwrap();
    for blocks in parse(&buf).into_iter() {
        match blocks {
            CodeOrData::Code(blocks) => {
                for code in blocks.into_iter() {
                    writeln!(output, "{}", code).unwrap();
                }
            }
            CodeOrData::Data(blocks) => {
                write!(output, "::core::write!(\n    output,\n    \"").unwrap();
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
                        DataSection::Raw(s) => {
                            writeln!(output, "    &({}),", s).unwrap();
                        }
                        DataSection::Escaped(s) => {
                            writeln!(output, "    ::nate::XmlEscape(&({})),", s).unwrap();
                        }
                        DataSection::Debug(s) => {
                            writeln!(output, "    &({}),", s).unwrap();
                        }
                        DataSection::Verbose(s) => {
                            writeln!(output, "    &({}),", s).unwrap();
                        }
                    }
                }
                writeln!(output, "\n)?;").unwrap();
            }
        }
    }
}

fn parse(i: &[u8]) -> Vec<CodeOrData<'_>> {
    WsBlockIter::new(i)
        .map(|WsBlock(a, b, z)| match b {
            Block::Data(s) => Block::Data(match (a, z) {
                (true, true) => s.trim(),
                (true, false) => s.trim_start(),
                (false, true) => s.trim_end(),
                (false, false) => s,
            }),
            b => b,
        })
        .fold(Vec::new(), |mut accu, block| {
            match block {
                Block::Comment(_) => {}
                Block::Code(s) => match accu.last_mut() {
                    Some(CodeOrData::Code(blocks)) => blocks.push(s),
                    _ => accu.push(CodeOrData::Code(vec![s])),
                },
                Block::Data(s)
                | Block::Raw(s)
                | Block::Escaped(s)
                | Block::Debug(s)
                | Block::Verbose(s) => {
                    let data = match block {
                        Block::Data(_) => DataSection::Data(s),
                        Block::Raw(_) => DataSection::Raw(s),
                        Block::Escaped(_) => DataSection::Escaped(s),
                        Block::Debug(_) => DataSection::Debug(s),
                        Block::Verbose(_) => DataSection::Verbose(s),
                        _ => panic!("Impossible"),
                    };
                    match accu.last_mut() {
                        Some(CodeOrData::Data(blocks)) => blocks.push(data),
                        _ => accu.push(CodeOrData::Data(vec![data])),
                    }
                }
            }
            accu
        })
}

enum CodeOrData<'a> {
    Code(Vec<&'a str>),
    Data(Vec<DataSection<'a>>),
}

#[derive(Clone)]
enum DataSection<'a> {
    Data(&'a str),
    Raw(&'a str),
    Escaped(&'a str),
    Debug(&'a str),
    Verbose(&'a str),
}

#[derive(Clone)]
enum Block<'a> {
    Data(&'a str),
    Raw(&'a str),
    Escaped(&'a str),
    Debug(&'a str),
    Verbose(&'a str),
    Code(&'a str),
    Comment(&'a [u8]),
}

struct WsBlock<'a>(bool, Block<'a>, bool);

struct WsBlockIter<'a>(&'a [u8], Option<WsBlock<'a>>);

impl<'a> WsBlockIter<'a> {
    fn new(mut i: &'a [u8]) -> Self {
        let b = parse_ws_block(&mut i);
        Self(i, b)
    }
}

impl<'a> Iterator for WsBlockIter<'a> {
    type Item = WsBlock<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match (parse_ws_block(&mut self.0), &self.1) {
            (Some(WsBlock(next_a, next_b, next_z)), Some(WsBlock(cur_a, cur_b, cur_z))) => {
                let result = WsBlock(*cur_a, cur_b.clone(), cur_z | next_a);
                self.1 = Some(WsBlock(next_a || *cur_z, next_b, next_z));
                Some(result)
            }
            (None, _) => self.1.take(),
            (cur, None) => cur, // impossible
        }
    }
}

fn parse_ws_block<'a>(i: &mut &'a [u8]) -> Option<WsBlock<'a>> {
    if !i.is_empty() {
        let (j, b) = alt((
            parse_ws_verbose,
            parse_ws_debug,
            parse_ws_raw,
            parse_ws_escaped,
            parse_ws_comment,
            parse_ws_code,
            parse_ws_data,
        ))(*i)
        .unwrap();
        *i = j;
        Some(b)
    } else {
        None
    }
}

fn parse_ws_data(i: &[u8]) -> IResult<&[u8], WsBlock<'_>> {
    let (i, b) = parse_data(i)?;
    Ok((i, WsBlock(false, b, false)))
}

fn parse_ws_verbose(i: &[u8]) -> IResult<&[u8], WsBlock<'_>> {
    let (i, (a, b, z)) = preceded(
        tag(b"{{{{{"),
        cut(terminated(
            tuple((opt(tag(b"-")), parse_verbose, opt(tag(b"-")))),
            tag(b"}}}}}"),
        )),
    )(i)?;
    Ok((i, WsBlock(a.is_some(), b, z.is_some())))
}

fn parse_ws_debug(i: &[u8]) -> IResult<&[u8], WsBlock<'_>> {
    let (i, (a, b, z)) = preceded(
        tag(b"{{{{"),
        cut(terminated(
            tuple((opt(tag(b"-")), parse_debug, opt(tag(b"-")))),
            tag(b"}}}}"),
        )),
    )(i)?;
    Ok((i, WsBlock(a.is_some(), b, z.is_some())))
}

fn parse_ws_raw(i: &[u8]) -> IResult<&[u8], WsBlock<'_>> {
    let (i, (a, b, z)) = preceded(
        tag(b"{{{"),
        cut(terminated(
            tuple((opt(tag(b"-")), parse_raw, opt(tag(b"-")))),
            tag(b"}}}"),
        )),
    )(i)?;
    Ok((i, WsBlock(a.is_some(), b, z.is_some())))
}

fn parse_ws_escaped(i: &[u8]) -> IResult<&[u8], WsBlock<'_>> {
    let (i, (a, b, z)) = preceded(
        tag(b"{{"),
        cut(terminated(
            tuple((opt(tag(b"-")), parse_escaped, opt(tag(b"-")))),
            tag(b"}}"),
        )),
    )(i)?;
    Ok((i, WsBlock(a.is_some(), b, z.is_some())))
}

fn parse_ws_comment(i: &[u8]) -> IResult<&[u8], WsBlock<'_>> {
    let (i, (a, b, z)) = preceded(
        tag(b"{#"),
        cut(tuple((
            opt(tag(b"-")),
            parse_comment,
            opt(terminated(opt(tag(b"-")), tag(b"#}"))),
        ))),
    )(i)?;
    Ok((i, WsBlock(a.is_some(), b, z.is_some())))
}

fn parse_ws_code(i: &[u8]) -> IResult<&[u8], WsBlock<'_>> {
    let (i, (a, b, z)) = preceded(
        tag(b"{%"),
        cut(tuple((
            opt(tag(b"-")),
            parse_code,
            opt(terminated(opt(tag(b"-")), tag(b"%}"))),
        ))),
    )(i)?;
    Ok((i, WsBlock(a.is_some(), b, z.is_some())))
}

fn parse_comment(i: &[u8]) -> IResult<&[u8], Block<'_>> {
    fn end(i: &[u8]) -> IResult<&[u8], &[u8]> {
        alt((tag(b"-#}"), tag("#}")))(i)
    }
    if i.len() >= 2 {
        let mut start = 0;
        while start < i.len() {
            match memchr2(b'#', b'-', &i[start..=i.len() - 2]) {
                Some(pos) if end(&i[pos..]).is_ok() => {
                    return Ok((&i[start + pos..], Block::Comment(&i[..start + pos])))
                }
                Some(pos) => {
                    start += pos;
                }
                None => {
                    break;
                }
            }
        }
    }
    Ok((b"", Block::Comment(i)))
}

fn parse_verbose(i: &[u8]) -> IResult<&[u8], Block<'_>> {
    fn end(i: &[u8]) -> IResult<&[u8], &[u8]> {
        alt((tag(b"-}}}}}"), tag("}}}}}")))(i)
    }
    if i.len() >= 5 {
        let mut start = 0;
        while start < i.len() {
            match memchr2(b'}', b'-', &i[start..=i.len() - 5]) {
                Some(pos) if end(&i[pos..]).is_ok() => {
                    return Ok((
                        &i[start + pos..],
                        Block::Verbose(from_utf8(&i[..start + pos]).unwrap()),
                    ))
                }
                Some(pos) => {
                    start += pos;
                }
                None => {
                    break;
                }
            }
        }
    }
    Err(nom::Err::Error(nom::error::Error {
        input: i,
        code: nom::error::ErrorKind::Tag,
    }))
}

fn parse_debug(i: &[u8]) -> IResult<&[u8], Block<'_>> {
    fn end(i: &[u8]) -> IResult<&[u8], &[u8]> {
        alt((tag(b"-}}}}"), tag("}}}}")))(i)
    }
    if i.len() >= 4 {
        let mut start = 0;
        while start < i.len() {
            match memchr2(b'}', b'-', &i[start..=i.len() - 4]) {
                Some(pos) if end(&i[pos..]).is_ok() => {
                    return Ok((
                        &i[start + pos..],
                        Block::Debug(from_utf8(&i[..start + pos]).unwrap()),
                    ))
                }
                Some(pos) => {
                    start += pos;
                }
                None => {
                    break;
                }
            }
        }
    }
    Err(nom::Err::Error(nom::error::Error {
        input: i,
        code: nom::error::ErrorKind::Tag,
    }))
}

fn parse_raw(i: &[u8]) -> IResult<&[u8], Block<'_>> {
    fn end(i: &[u8]) -> IResult<&[u8], &[u8]> {
        alt((tag(b"-}}}"), tag("}}}")))(i)
    }
    if i.len() >= 3 {
        let mut start = 0;
        while start < i.len() {
            match memchr2(b'}', b'-', &i[start..=i.len() - 3]) {
                Some(pos) if end(&i[pos..]).is_ok() => {
                    return Ok((
                        &i[start + pos..],
                        Block::Raw(from_utf8(&i[..start + pos]).unwrap()),
                    ))
                }
                Some(pos) => {
                    start += pos;
                }
                None => {
                    break;
                }
            }
        }
    }
    Err(nom::Err::Error(nom::error::Error {
        input: i,
        code: nom::error::ErrorKind::Tag,
    }))
}

fn parse_escaped(i: &[u8]) -> IResult<&[u8], Block<'_>> {
    fn end(i: &[u8]) -> IResult<&[u8], &[u8]> {
        alt((tag(b"-}}"), tag("}}")))(i)
    }
    if i.len() >= 2 {
        let mut start = 0;
        while start < i.len() {
            match memchr2(b'}', b'-', &i[start..=i.len() - 2]) {
                Some(pos) if end(&i[pos..]).is_ok() => {
                    return Ok((
                        &i[start + pos..],
                        Block::Escaped(from_utf8(&i[..start + pos]).unwrap()),
                    ))
                }
                Some(pos) => {
                    start += pos;
                }
                None => {
                    break;
                }
            }
        }
    }
    Err(nom::Err::Error(nom::error::Error {
        input: i,
        code: nom::error::ErrorKind::Tag,
    }))
}

fn parse_code(i: &[u8]) -> IResult<&[u8], Block<'_>> {
    fn end(i: &[u8]) -> IResult<&[u8], &[u8]> {
        alt((tag(b"-%}"), tag("%}")))(i)
    }
    if i.len() >= 2 {
        let mut start = 0;
        while start < i.len() {
            match memchr2(b'%', b'-', &i[start..=i.len() - 2]) {
                Some(pos) if end(&i[pos..]).is_ok() => {
                    return Ok((
                        &i[start + pos..],
                        Block::Code(from_utf8(&i[..start + pos]).unwrap()),
                    ))
                }
                Some(pos) => {
                    start += pos;
                }
                None => {
                    break;
                }
            }
        }
    }
    Ok((b"", Block::Code(from_utf8(i).unwrap())))
}

fn parse_data(i: &[u8]) -> IResult<&[u8], Block<'_>> {
    let (i, _) = not(eof)(i)?;
    match memchr(b'{', i) {
        Some(pos) => Ok((&i[pos..], Block::Data(from_utf8(&i[..pos]).unwrap()))),
        None => Ok((b"", Block::Data(from_utf8(i).unwrap()))),
    }
}
