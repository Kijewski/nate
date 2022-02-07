use std::env::var;
use std::fmt::Write;
use std::fs::OpenOptions;
use std::io::{Read, Write as _};
use std::path::{Path, PathBuf};

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::compile_error::CompileError;
use crate::parse::{input_into_blocks, Block, DataSection};
use crate::span_data::{input_only, input_with_path, SpanInput};
use crate::TemplateAttrs;

#[derive(Debug)]
enum ParsedData {
    Code(Vec<SpanInput>),
    Data(Vec<DataSection>),
}

const TAIL: &str = r#"
            ::nate::details::std::fmt::Result::Ok(())
        }
    }
};
"#;

pub(crate) fn generate(input: TokenStream) -> Result<TokenStream, CompileError> {
    let ast: DeriveInput = syn::parse(input)?;
    let ident = ast.ident.to_string();

    let opts = TemplateAttrs::from_derive_input(&ast)?;

    let base = var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&base).join(&opts.path);
    let output = opts.generated.as_ref().map(|s| Path::new(&base).join(s));

    let left_generics = &ast.generics;
    let left_generics = quote!(#left_generics);

    let mut generics = String::new();
    if !ast.generics.params.is_empty() {
        write!(generics, "<")?;
        for arg in ast.generics.params.iter() {
            match arg {
                syn::GenericParam::Type(ty) => {
                    write!(generics, " {}, ", ty.ident)?;
                },
                syn::GenericParam::Lifetime(def) => {
                    write!(generics, " '{}, ", def.lifetime.ident)?;
                },
                syn::GenericParam::Const(par) => {
                    write!(generics, " {}, ", par.ident)?;
                },
            }
        }
        writeln!(generics, ">")?;
    }
    if let Some(where_clause) = ast.generics.where_clause {
        writeln!(generics, " {} ", quote!(#where_clause))?;
    }

    let mut content = String::new();
    write!(
        content,
        r#"
#[automatically_derived]
#[allow(unused_qualifications)]
const _: () = {{
    impl {left_generics} ::nate::details::std::fmt::Display for {ident} {generics} {{
        #[inline]
        fn fmt(
            &self,
            output: &mut ::nate::details::std::fmt::Formatter<'_>,
        ) -> ::nate::details::std::fmt::Result {{
            ::nate::RenderInto::render_fmt(self, output)
        }}
    }}

    impl {left_generics} ::nate::RenderInto for {ident} {generics} {{
        fn render_into(
            &self,
            mut output: impl ::nate::WriteAny,
        ) -> ::nate::details::std::fmt::Result {{
            #[allow(unused_imports)]
            use ::nate::details::{{RawKind as _, EscapeKind as _}};
"#,
        left_generics = left_generics,
        generics = generics,
        ident = ident
    )?;

    parse_file(path, &mut content, &opts)?;
    write!(content, "{}", TAIL)?;

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

    Ok(content.parse()?)
}

fn load_file(path: &Path) -> String {
    let mut buf = String::new();
    match OpenOptions::new().read(true).open(&path) {
        Ok(mut f) => {
            let _ = f
                .read_to_string(&mut buf)
                .expect("Could not read source file even after successfully opening it.");
        },
        Err(err) => {
            eprintln!("Could not open file {:?}: {:?}", path, err);
            panic!();
        },
    }
    buf
}

fn push_address(span: &SpanInput, output: &mut impl Write) -> Result<(), CompileError> {
    let path = match span.path() {
        Some(path) => path.as_os_str(),
        None => return Ok(()),
    };
    writeln!(
        output,
        "// #[__nate_addr(path={path:?}, offset={offset:?}, row={row:?}, col={col:?})]",
        path = path,
        offset = span.location_offset(),
        row = span.location_line(),
        col = span.naive_get_utf8_column(),
    )?;
    Ok(())
}

fn parse_file(
    path: PathBuf,
    mut output: impl Write,
    opts: &TemplateAttrs,
) -> Result<(), CompileError> {
    use DataSection::*;

    let i = load_file(&path);
    for (block_index, blocks) in parse(path, i, opts)?.into_iter().enumerate() {
        match blocks {
            ParsedData::Code(blocks) => {
                for code in blocks.into_iter() {
                    push_address(&code, &mut output)?;
                    writeln!(output, "{}", code.as_str())?;
                }
            },
            ParsedData::Data(blocks) => {
                match &blocks[0] {
                    Data(s) | Raw(s) | Escaped(s) | Debug(s) | Verbose(s) => {
                        push_address(s, &mut output)?;
                    },
                }
                writeln!(output, "{{")?;

                for (data_index, data) in blocks.iter().enumerate() {
                    match data {
                        Data(_) => continue,
                        Raw(s) | Escaped(s) | Debug(s) | Verbose(s) => {
                            push_address(s, &mut output)?;
                            writeln!(
                                output,
                                "let _nate_{block}_{data} = &({expr});",
                                block = block_index,
                                data = data_index,
                                expr = s.as_str(),
                            )?;
                        },
                    }

                    match data {
                        Data(_) | Raw(_) => {},
                        Escaped(_) => {
                            writeln!(
                                output,
                                "let _nate_{block}_{data} = \
                                    (&::nate::details::TagWrapper::new(_nate_{block}_{data})).\
                                    wrap(_nate_{block}_{data});",
                                block = block_index,
                                data = data_index,
                            )?;
                        },
                        Debug(_) | Verbose(_) => {
                            writeln!(
                                output,
                                "let _nate_{block}_{data} = \
                                    ::nate::XmlEscape(_nate_{block}_{data});",
                                block = block_index,
                                data = data_index,
                            )?;
                        },
                    }
                }

                match &blocks[0] {
                    Data(s) | Raw(s) | Escaped(s) | Debug(s) | Verbose(s) => {
                        push_address(s, &mut output)?;
                    },
                }
                write!(
                    output,
                    "output.write_fmt(::nate::details::std::format_args!(\n\""
                )?;
                for (data_index, data) in blocks.iter().enumerate() {
                    match data {
                        Data(s) => {
                            let s = format!("{:#?}", s.as_str())
                                .replace('{', "{{")
                                .replace('}', "}}");
                            write!(output, "{}", &s[1..s.len() - 1])?;
                        },
                        Raw(_) | Escaped(_) => write!(
                            output,
                            "{{_nate_{block}_{data}}}",
                            block = block_index,
                            data = data_index
                        )?,
                        Debug(_) => write!(
                            output,
                            "{{_nate_{block}_{data}:?}}",
                            block = block_index,
                            data = data_index
                        )?,
                        Verbose(_) => write!(
                            output,
                            "{{_nate_{block}_{data}:#?}}",
                            block = block_index,
                            data = data_index
                        )?,
                    }
                }

                writeln!(output, "\",")?;
                for (data_index, data) in blocks.into_iter().enumerate() {
                    match data {
                        Data(_) => {},
                        Raw(_) | Escaped(_) | Debug(_) | Verbose(_) => writeln!(
                            output,
                            "_nate_{block}_{data} = _nate_{block}_{data},",
                            block = block_index,
                            data = data_index
                        )?,
                    }
                }
                writeln!(output, "))?;\n}}")?;
            },
        }
    }

    Ok(())
}

fn parse(path: PathBuf, i: String, opts: &TemplateAttrs) -> Result<Vec<ParsedData>, CompileError> {
    let mut output = Vec::new();
    parse_into(path, i, &mut output, opts)?;
    Ok(output)
}

fn parse_into(
    path: PathBuf,
    i: String,
    accu: &mut Vec<ParsedData>,
    opts: &TemplateAttrs,
) -> Result<(), CompileError> {
    let it = input_into_blocks(input_with_path(i.into(), path.clone().into()));

    let s = input_only(format!(
        "\
{{\n\
const _: &'static [::nate::details::std::primitive::u8] = \
::nate::details::std::include_bytes!({:?});",
        path
    ));
    match accu.last_mut() {
        Some(ParsedData::Code(blocks)) => blocks.push(s),
        _ => accu.push(ParsedData::Code(vec![s])),
    }

    for block in it {
        match block? {
            Block::Comment => {},
            Block::Code(s) => match accu.last_mut() {
                Some(ParsedData::Code(blocks)) => blocks.push(s),
                _ => accu.push(ParsedData::Code(vec![s])),
            },
            Block::Data(data) => match accu.last_mut() {
                Some(ParsedData::Data(blocks)) => blocks.push(data),
                _ => accu.push(ParsedData::Data(vec![data])),
            },
            Block::Include(include_path) => {
                let include_path = include_path.as_str().trim();
                let include_path = match Path::new(include_path).iter().next() {
                    Some(d) if d.eq(".") || d.eq("..") => {
                        path.parent().unwrap_or(&path).join(include_path)
                    },
                    _ => Path::new(&var("CARGO_MANIFEST_DIR").map_err(|_| std::fmt::Error)?)
                        .join(include_path),
                };
                let buf = load_file(&include_path);
                parse_into(include_path, buf, accu, opts)?;
            },
        }
    }

    let s = input_only("}");
    match accu.last_mut() {
        Some(ParsedData::Code(blocks)) => blocks.push(s),
        _ => accu.push(ParsedData::Code(vec![s])),
    }

    Ok(())
}
