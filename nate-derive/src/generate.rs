use std::borrow::Cow;
use std::convert::TryInto;
use std::env::var;
use std::fmt::{self, Write};
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

    #[allow(unknown_lints)]
    #[allow(unused_qualifications)]
    #[allow(clippy::needless_borrow)]
    #[allow(clippy::needless_borrowed_reference)]
    #[allow(clippy::suspicious_else_formatting)]
    impl {impl_generics} ::nate::RenderInto
        for {ident} {type_generics} {where_clause}
    {{
        fn render_into(
            &self,
            mut output: impl ::nate::WriteAny,
        ) -> ::nate::details::std::fmt::Result {{
"#,
        impl_generics = quote!(#impl_generics),
        type_generics = quote!(#type_generics),
        where_clause = quote!(#where_clause),
        ident = quote!(#ident),
    )?;
    parse_file(path, &mut content, &mut ctx)?;
    write!(content, "{}", TAIL)?;
    let content = content.as_str();

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

    let f = OpenOptions::new().write(true).create(true).open(&output);
    let mut f = match f {
        Ok(f) => f,
        Err(err) => return Err(CompileError::IoError(IoOp::Open, output, err)),
    };
    let exists = match f.metadata() {
        Ok(meta) => meta.len().try_into() == Ok(content.len()),
        Err(err) => return Err(CompileError::IoError(IoOp::Metadata, output, err)),
    };
    if !exists {
        if let Err(err) = f.set_len(0) {
            return Err(CompileError::IoError(IoOp::Write, output, err));
        }
        if let Err(err) = f.write_all(content.as_bytes()) {
            return Err(CompileError::IoError(IoOp::Write, output, err));
        }
    }
    drop(f);

    let output = output.to_str().unwrap();
    let content = quote! {
        const _: () = ::core::include!(#output);
    };
    Ok(content.into())
}

fn parse_file(
    path: PathBuf,
    mut output: impl Write,
    ctx: &mut Context,
) -> Result<(), CompileError> {
    use DataSection::*;

    let i = ctx.load_file(&path)?;
    for (block_index, blocks) in parse(path, i, ctx)?.into_iter().enumerate() {
        let blocks = match blocks {
            ParsedData::Code(blocks) => {
                for code in blocks.into_iter() {
                    writeln!(output, "/* {} */", AddrAnnotation(&code))?;
                    writeln!(output, "{}", code.as_str())?;
                }
                continue;
            },
            ParsedData::Data(blocks) => blocks,
        };
        let blocks = &blocks[..];

        if let [DataSection::Data(s)] = blocks {
            writeln!(output, "{{")?;
            writeln!(output, "/* {} */", AddrAnnotation(s))?;
            let s = format!("{:#?}", s.as_str())
                .replace('{', "{{")
                .replace('}', "}}");
            writeln!(output, "    <_ as ::nate::WriteAny>::write_str(")?;
            writeln!(output, "        &mut output,")?;
            writeln!(output, "        \"{}\",", &s[1..s.len() - 1])?;
            writeln!(output, "    )?;")?;
            writeln!(output, "}}")?;
            continue;
        }

        writeln!(output, "{{")?;

        let has_non_data = blocks.iter().any(|data| !matches!(data, Data(_)));
        let has_non_data_non_raw =
            has_non_data && blocks.iter().any(|data| !matches!(data, Data(_) | Raw(_)));

        if has_non_data {
            // let (_nate_X_Y, …) = (&(expr), …);
            writeln!(output, "    let (")?;
            for (data_index, data) in blocks.iter().enumerate() {
                if !matches!(data, Data(_)) {
                    writeln!(
                        output,
                        "        _nate_{block}_{data},",
                        block = block_index,
                        data = data_index,
                    )?;
                }
            }
            writeln!(output, "    ) = (")?;
            for data in blocks {
                match data {
                    Data(_) => {},
                    Raw(s) | Escaped(s) | Debug(s) | Verbose(s) => {
                        writeln!(output, "        /* {} */", AddrAnnotation(s))?;
                        writeln!(output, "        &({expr}),", expr = s.as_str())?;
                    },
                }
            }
            writeln!(output, "    );")?;
        }

        if has_non_data_non_raw {
            writeln!(output, "    {{")?;
            writeln!(output, "        #[allow(unused_imports)]")?;
            writeln!(output, "        use ::nate::details::{{")?;
            writeln!(output, "            EscapeKind as _,")?;
            writeln!(output, "            FloatKind as _,")?;
            writeln!(output, "            IntKind as _,")?;
            writeln!(output, "            RawKind as _,")?;
            writeln!(output, "        }};")?;

            // let (_nate_X_Y, …) = (&(EscapeWrapper::new(…)).wrap(…), …);
            writeln!(output, "        let (")?;
            for (data_index, data) in blocks.iter().enumerate() {
                if !matches!(data, Data(_) | Raw(_)) {
                    writeln!(
                        output,
                        "            _nate_{block}_{data},",
                        block = block_index,
                        data = data_index,
                    )?;
                }
            }
            writeln!(output, "        ) = (")?;
            for (data_index, data) in blocks.iter().enumerate() {
                match data {
                    Data(_) | Raw(_) => {},
                    Escaped(s) => {
                        writeln!(output, "            /* {} */", AddrAnnotation(s))?;
                        writeln!(
                            output,
                            "            (&::nate::details::EscapeWrapper::new(_nate_{block}_{data})).\
                                wrap(_nate_{block}_{data}),",
                            block = block_index,
                            data = data_index,
                        )?;
                    },
                    Debug(s) | Verbose(s) => {
                        writeln!(output, "            /* {} */", AddrAnnotation(s))?;
                        writeln!(
                            output,
                            "            ::nate::details::XmlEscape(_nate_{block}_{data}),",
                            block = block_index,
                            data = data_index,
                        )?;
                    },
                }
            }
            writeln!(output, "        );")?;
        }

        // write!(…);
        writeln!(output, "        <_ as ::nate::WriteAny>::write_fmt(")?;
        writeln!(output, "            &mut output,")?;
        writeln!(output, "            ::nate::details::std::format_args!(")?;
        write!(output, "                \"")?;
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
        for (data_index, data) in blocks.iter().enumerate() {
            if let Data(_) = data {
                continue;
            };
            writeln!(
                output,
                "                _nate_{block}_{data} = _nate_{block}_{data},",
                block = block_index,
                data = data_index,
            )?;
        }
        writeln!(output, "            ),")?;
        writeln!(output, "        )?;")?;
        if has_non_data_non_raw {
            writeln!(output, "    }}")?;
        }
        writeln!(output, "}}")?;
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

struct AddrAnnotation<'a>(&'a SpanInput);

impl fmt::Display for AddrAnnotation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = match self.0.get_shared() {
            Some(path) => path.as_os_str(),
            None => return Ok(()),
        };
        write!(
            f,
            r"#[::nate::addr(path={path:?}, offset={offset:?}, row={row:?}, col={col:?})]",
            path = path,
            offset = self.0.location_offset(),
            row = self.0.location_line(),
            col = self.0.naive_get_utf8_column(),
        )
    }
}
