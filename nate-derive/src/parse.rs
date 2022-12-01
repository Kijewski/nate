use std::iter::Peekable;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::combinator::{cut, opt, rest};
use nom::error::ErrorKind;
use nom::sequence::{pair, preceded};
use nom::{error_position, IResult, InputTake};

use crate::compile_error::CompileError;
use crate::generate::SpanInput;

pub(crate) fn input_into_blocks(i: SpanInput) -> impl Iterator<Item = Result<Block, CompileError>> {
    WsBlockIter(BlockIter(Some(i)).peekable()).filter_map(|item| {
        let WsBlock(a, b, z) = match item {
            Ok(block) => block,
            Err(err) => return Some(Err(err)),
        };
        let b = match b {
            Block::Data(DataSection::Data(s)) => {
                let s = match (a, z) {
                    (true, true) => s.trim(),
                    (true, false) => s.trim_start(),
                    (false, true) => s.trim_end(),
                    (false, false) => s,
                };
                Block::Data(DataSection::Data(s))
            },
            b => b,
        };
        if b.is_empty() { None } else { Some(Ok(b)) }
    })
}

#[derive(Debug, Clone)]
pub(crate) enum DataSection {
    Data(SpanInput),
    Raw(SpanInput),
    Escaped(SpanInput),
    Debug(SpanInput),
    Verbose(SpanInput),
}

#[derive(Debug, Clone)]
pub(crate) enum Block {
    Data(DataSection),
    Code(SpanInput),
    Comment,
    Include(SpanInput),
}

#[derive(Debug)]
struct WsBlock(bool, Block, bool);

impl Block {
    fn is_empty(&self) -> bool {
        match self {
            Block::Comment => true,
            Block::Code(s) | Block::Data(DataSection::Data(s)) => s.is_empty(),
            Block::Data(_) | Block::Include(_) => false,
        }
    }
}

#[derive(Debug)]
struct BlockIter(Option<SpanInput>);

impl Iterator for BlockIter {
    type Item = Result<WsBlock, CompileError>;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.0.take()?;
        if i.is_empty() {
            None
        } else {
            match parse_ws_block(i) {
                Ok((i, block)) => {
                    self.0 = Some(i);
                    Some(Ok(block))
                },
                Err(err) => Some(Err(CompileError::Nom(err))),
            }
        }
    }
}

#[derive(Debug)]
struct WsBlockIter(Peekable<BlockIter>);

impl Iterator for WsBlockIter {
    type Item = Result<WsBlock, CompileError>;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.0.next()?, self.0.peek_mut()) {
            (Ok(mut cur), Some(Ok(next))) => {
                let trim = cur.2 || next.0;
                cur.2 = trim;
                next.0 = trim;
                Some(Ok(cur))
            },
            (cur, _) => Some(cur),
        }
    }
}

fn fail_if_empty(b: SpanInput) -> Result<SpanInput, nom::Err<nom::error::Error<SpanInput>>> {
    if b.is_empty() {
        Err(nom::Err::Failure(error_position!(b, ErrorKind::NonEmpty)))
    } else {
        Ok(b)
    }
}

fn parse_ws_block(i: SpanInput) -> IResult<SpanInput, WsBlock> {
    alt((
        |i| {
            let (i, (a, b, z)) = parse_block(i, "{%", "%}")?;
            Ok((i, WsBlock(a, Block::Code(b), z)))
        },
        |i| {
            let (i, (a, _, z)) = parse_block(i, "{#", "#}")?;
            Ok((i, WsBlock(a, Block::Comment, z)))
        },
        |i: SpanInput| {
            let (next_i, (a, b, z)) = parse_block(i, "{<", ">}")?;
            let b = fail_if_empty(b)?;
            Ok((next_i, WsBlock(a, Block::Include(b), z)))
        },
        |i| parse_data_section(i, "{{{{{", "}}}}}", DataSection::Verbose),
        |i| parse_data_section(i, "{{{{", "}}}}", DataSection::Debug),
        |i| parse_data_section(i, "{{{", "}}}", DataSection::Raw),
        |i| parse_data_section(i, "{{", "}}", DataSection::Escaped),
        parse_data,
    ))(i)
}

fn parse_block(
    i: SpanInput,
    start: &'static str,
    end: &'static str,
) -> IResult<SpanInput, (bool, SpanInput, bool)> {
    let inner = |i: SpanInput| -> IResult<SpanInput, (SpanInput, bool)> {
        let (i, inner) = opt(take_until(end))(i)?;
        let inner = if let Some(inner) = inner {
            inner
        } else {
            let (i, inner) = rest(i)?;
            return Ok((i, (inner, false)));
        };

        let (i, _) = i.take_split(end.len());
        let (inner, trim) = if (*inner).ends_with('-') {
            (inner.take(inner.len() - 1), true)
        } else {
            (inner, false)
        };

        let inner = inner.trim();
        Ok((i, (inner, trim)))
    };

    let (i, (trim_start, (b, trim_end))) =
        preceded(tag(start), cut(pair(opt(tag("-")), inner)))(i)?;
    Ok((i, (trim_start.is_some(), b, trim_end)))
}

fn parse_data_section(
    i: SpanInput,
    start: &'static str,
    end: &'static str,
    kind: impl 'static + Fn(SpanInput) -> DataSection,
) -> IResult<SpanInput, WsBlock> {
    let (j, (trim_start, b, trim_end)) = parse_block(i, start, end)?;
    let b = fail_if_empty(b)?;
    Ok((j, WsBlock(trim_start, Block::Data(kind(b)), trim_end)))
}

fn parse_data(i: SpanInput) -> IResult<SpanInput, WsBlock> {
    let mut offset = i.char_indices();

    // TODO: make pretty once `#![feature(let_chains_2)]` (RFC 2497) is ready
    // rationale: no need to use `let offset = offset.fuse()`
    if offset.next().is_some() {
        if let Some((offset, _)) = offset.next() {
            if let Some(inner) = opt(take_until("{"))(i.take_split(offset).0)?.1 {
                let (i, b) = i.take_split(inner.len() + offset);
                let b = DataSection::Data(b);
                return Ok((i, WsBlock(false, Block::Data(b), false)));
            }
        }
    }

    let (i, b) = rest(i)?;
    let b = DataSection::Data(b);
    Ok((i, WsBlock(false, Block::Data(b), false)))
}
