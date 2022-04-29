use std::borrow::Cow;
use std::env::var;
use std::fmt::Write;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use blake2::digest::FixedOutput;
use blake2::Digest;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::compile_error::{CompileError, IoOp};
use crate::nate_span::SpanStatic;
use crate::parse::{input_into_blocks, Block, DataSection};
use crate::{Context, Settings};

pub(crate) type SpanInput = SpanStatic<(), Option<Cow<'static, Path>>>;

#[derive(Debug)]
enum ParsedData {
    Code(Vec<SpanInput>),
    Data(Vec<DataSection>),
}

const TAIL: &str = r#"
            ::nate::details::std::fmt::Result::Ok(())
        }
    }
}
"#;

pub(crate) fn generate(input: TokenStream) -> Result<TokenStream, CompileError> {
    let ast: DeriveInput = syn::parse(input)?;
    let mut ctx = Context {
        settings: Settings::from_derive_input(&ast)?,
        ..Default::default()
    };

    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();
    let ident = ast.ident;

    let base = var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&base).join(&ctx.settings.path);
    let output = ctx
        .settings
        .generated
        .as_ref()
        .map(|s| Path::new(&base).join(s));

    let mut content = String::new();
    write!(
        content,
        r#"{{
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl {impl_generics} ::nate::details::std::fmt::Display
        for {ident} {type_generics} {where_clause}
    {{
        #[inline]
        fn fmt(
            &self,
            output: &mut ::nate::details::std::fmt::Formatter<'_>,
        ) -> ::nate::details::std::fmt::Result {{
            ::nate::RenderInto::render_fmt(self, output)
        }}
    }}

    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::suspicious_else_formatting)]
    impl {impl_generics} ::nate::RenderInto
        for {ident} {type_generics} {where_clause}
    {{
        fn render_into(
            &self,
            mut output: impl ::nate::WriteAny,
        ) -> ::nate::details::std::fmt::Result {{
            #[allow(unused_imports)]
            use ::nate::details::{{RawKind as _, EscapeKind as _}};
"#,
        impl_generics = quote!(#impl_generics),
        type_generics = quote!(#type_generics),
        where_clause = quote!(#where_clause),
        ident = quote!(#ident),
    )?;
    parse_file(path, &mut content, &mut ctx)?;
    write!(content, "{}", TAIL)?;

    let output = match output {
        Some(output) => output,
        None => {
            ctx.strings_hash.update(content.as_bytes());
            let out_dir = AsRef::<Path>::as_ref(&env!("NATE_DERIVE_OUTDIR"));
            let mut temp_name = hex::encode(ctx.strings_hash.finalize_fixed());
            temp_name.push_str(".rs");
            out_dir.join(temp_name)
        },
    };

    let f = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&output);
    let mut f = match f {
        Ok(f) => f,
        Err(err) => return Err(CompileError::IoError(IoOp::Open, output, err)),
    };
    if let Err(err) = write!(f, "{}", &content) {
        return Err(CompileError::IoError(IoOp::Write, output, err));
    }

    let output = output.to_str().unwrap();
    let content = quote! {
        const _: () = ::core::include!(#output);
    };
    Ok(content.into())
}

fn push_address(span: &SpanInput, output: &mut impl Write) -> Result<(), CompileError> {
    let path = match span.get_shared() {
        Some(path) => path.as_os_str(),
        None => return Ok(()),
    };
    writeln!(
        output,
        r"// #[::nate::addr(path={path:?}, offset={offset:?}, row={row:?}, col={col:?})]",
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
    ctx: &mut Context,
) -> Result<(), CompileError> {
    use DataSection::*;

    let i = ctx.load_file(&path)?;
    for (block_index, blocks) in parse(path, i, ctx)?.into_iter().enumerate() {
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
                                "    let _nate_{block}_{data} = &({expr});",
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
                                "    let _nate_{block}_{data} = \
                                    (&::nate::details::TagWrapper::new(_nate_{block}_{data})).\
                                    wrap(_nate_{block}_{data});",
                                block = block_index,
                                data = data_index,
                            )?;
                        },
                        Debug(_) | Verbose(_) => {
                            writeln!(
                                output,
                                "    let _nate_{block}_{data} = \
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

                writeln!(output, "    <_ as ::nate::WriteAny>::write_fmt(")?;
                writeln!(output, "        &mut output,")?;
                writeln!(output, "        ::nate::details::std::format_args!(")?;
                write!(output, "            \"")?;
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
                            "            _nate_{block}_{data} = _nate_{block}_{data},",
                            block = block_index,
                            data = data_index
                        )?,
                    }
                }
                writeln!(output, "        ),")?;
                writeln!(output, "    )?;")?;
                writeln!(output, "}}")?;
            },
        }
    }

    Ok(())
}

fn parse(path: PathBuf, i: String, ctx: &mut Context) -> Result<Vec<ParsedData>, CompileError> {
    let mut output = Vec::new();
    parse_into(path, i, &mut output, ctx)?;
    Ok(output)
}

fn parse_into(
    path: PathBuf,
    i: String,
    accu: &mut Vec<ParsedData>,
    ctx: &mut Context,
) -> Result<(), CompileError> {
    let span = SpanInput::new_with_shared(i, Some(path.into()));
    let path = span.get_rc();
    let path = path.1.as_ref().unwrap().as_ref();

    let s = SpanInput::new(format!(
        "\
{{\n\
const _: &[::nate::details::std::primitive::u8] = \
::nate::details::std::include_bytes!({:?});",
        path
    ));
    match accu.last_mut() {
        Some(ParsedData::Code(blocks)) => blocks.push(s),
        _ => accu.push(ParsedData::Code(vec![s])),
    }

    for block in input_into_blocks(span) {
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
                        path.parent().unwrap_or(path).join(include_path)
                    },
                    _ => Path::new(&var("CARGO_MANIFEST_DIR").map_err(|_| std::fmt::Error)?)
                        .join(include_path),
                };
                let buf = ctx.load_file(&include_path)?;
                parse_into(include_path, buf, accu, ctx)?;
            },
        }
    }

    let s = SpanInput::new("}");
    match accu.last_mut() {
        Some(ParsedData::Code(blocks)) => blocks.push(s),
        _ => accu.push(ParsedData::Code(vec![s])),
    }

    Ok(())
}
