use std::fmt::{Display, Formatter};
use std::path::Path;

use nom::Offset;

use crate::span_data::SpanInput;

#[derive(Debug)]
pub(crate) enum CompileError {
    Nom(nom::Err<nom::error::Error<SpanInput>>),
    Darling(darling::Error),
    Syn(syn::Error),
    Lex(proc_macro::LexError),
    Fmt(std::fmt::Error),
}

impl std::error::Error for CompileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CompileError::Nom(err) => Some(err),
            CompileError::Darling(err) => Some(err),
            CompileError::Syn(err) => Some(err),
            CompileError::Lex(err) => Some(err),
            CompileError::Fmt(err) => Some(err),
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
            CompileError::Fmt(_) => return write!(f, "Could not format generated code"),
            CompileError::Darling(err) => return write!(f, "{}", err),
            CompileError::Syn(err) => return write!(f, "{}", err),
            CompileError::Lex(err) => return write!(f, "Could not parse input: {}", err),
            CompileError::Nom(err) => err,
        };

        let input = match err {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(err) | nom::Err::Failure(err) => &err.input,
        };
        let (source, path) = input.get_data();
        let path = path.and_then(Path::to_str).unwrap_or("??");
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
