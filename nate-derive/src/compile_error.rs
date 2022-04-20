use std::env::current_dir;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use nom::Offset;

use crate::generate::SpanInput;

#[derive(Debug)]
pub(crate) enum CompileError {
    Nom(nom::Err<nom::error::Error<SpanInput>>),
    Darling(darling::Error),
    Syn(syn::Error),
    Lex(proc_macro::LexError),
    Fmt(std::fmt::Error),
    IoError(IoOp, PathBuf, std::io::Error),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum IoOp {
    Open,
    Read,
    Write,
}

impl std::error::Error for CompileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CompileError::Nom(err) => Some(err),
            CompileError::Darling(err) => Some(err),
            CompileError::Syn(err) => Some(err),
            CompileError::Lex(err) => Some(err),
            CompileError::Fmt(err) => Some(err),
            CompileError::IoError(_, _, err) => Some(err),
        }
    }
}

impl From<nom::Err<nom::error::Error<SpanInput>>> for CompileError {
    fn from(err: nom::Err<nom::error::Error<SpanInput>>) -> Self {
        Self::Nom(err)
    }
}

impl From<darling::Error> for CompileError {
    fn from(err: darling::Error) -> Self {
        Self::Darling(err)
    }
}

impl From<syn::Error> for CompileError {
    fn from(err: syn::Error) -> Self {
        Self::Syn(err)
    }
}

impl From<proc_macro::LexError> for CompileError {
    fn from(err: proc_macro::LexError) -> Self {
        Self::Lex(err)
    }
}

impl From<std::fmt::Error> for CompileError {
    fn from(err: std::fmt::Error) -> Self {
        Self::Fmt(err)
    }
}

impl Display for CompileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err = match self {
            CompileError::Fmt(_) => return write!(f, "could not format generated code"),
            CompileError::Darling(err) => return write!(f, "{}", err),
            CompileError::Syn(err) => return write!(f, "{}", err),
            CompileError::Lex(err) => return write!(f, "could not parse input: {}", err),
            CompileError::IoError(op, path, err) => {
                let path = current_dir()
                    .ok()
                    .and_then(|cwd| path.strip_prefix(cwd).ok())
                    .unwrap_or(path);
                let op = match op {
                    IoOp::Open => "open",
                    IoOp::Read => "read from",
                    IoOp::Write => "write to",
                };
                return write!(f, "could not {} {:?}: {}", op, path, err);
            },
            CompileError::Nom(err) => err,
        };

        let input = match err {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(err) | nom::Err::Failure(err) => &err.input,
        };
        let source = input.get_source();
        let path = input
            .get_shared()
            .as_deref()
            .and_then(|p| p.to_str())
            .unwrap_or("??");
        let row = input.location_line();
        let column = input.naive_get_utf8_column();

        let source_after = &source[source.offset(input)..];
        let source_after = match source_after.char_indices().enumerate().take(73).last() {
            Some((72, (i, _))) => format!("{:?}...", &source_after[..i]),
            _ => format!("{:?}", source_after),
        };

        write!(
            f,
            "Problems parsing template source {:?} at row {}, column {} near:\n{}",
            path, row, column, source_after,
        )
    }
}
