use std::borrow::Cow;
use std::ops::{Deref, Range, RangeFrom, RangeFull, RangeTo};
use std::path::Path;
use std::rc::Rc;

use nom::{AsBytes, Compare, CompareResult, FindSubstring, InputLength, Offset, Slice};
use nom_locate::LocatedSpan;

#[derive(Debug)]
struct Data {
    string: Cow<'static, str>,
    path: Option<Cow<'static, Path>>,
}

#[derive(Debug, Clone)]
pub(crate) struct SpanData {
    start: usize,
    end: usize,
    data: Rc<Data>,
}

impl SpanData {
    pub(crate) fn build(string: Cow<'static, str>, path: Option<Cow<'static, Path>>) -> Self {
        Self {
            start: 0,
            end: string.len(),
            data: Rc::new(Data { string, path }),
        }
    }

    fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn get_data(&self) -> (&str, Option<&Path>) {
        let data = &*self.data;
        (data.string.as_ref(), data.path.as_deref())
    }

    pub(crate) fn offset(&self) -> usize {
        self.start
    }

    pub(crate) fn range(&self) -> Range<usize> {
        self.start..self.start + self.len()
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.data.string.as_ref()[self.range()]
    }

    pub(crate) fn trim_start_range(&self) -> RangeFrom<usize> {
        trim_start_range(self.as_str())
    }

    #[allow(unused)]
    pub(crate) fn trim_end_range(&self) -> RangeTo<usize> {
        trim_end_range(self.as_str())
    }

    pub(crate) fn trim_range(&self) -> Range<usize> {
        trim_range(self.as_str())
    }

    pub(crate) fn path(&self) -> Option<&Path> {
        self.data.path.as_deref()
    }
}

fn trim_start_range(i: &str) -> RangeFrom<usize> {
    i.len() - i.trim_start().len()..
}

fn trim_end_range(i: &str) -> RangeTo<usize> {
    ..i.trim_end().len()
}

fn trim_range(i: &str) -> Range<usize> {
    let trimmed = i.trim_start();
    let start = i.len() - trimmed.len();
    start..start + trimmed.trim_end().len()
}

impl PartialEq for SpanData {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl AsBytes for SpanData {
    fn as_bytes(&self) -> &[u8] {
        &self.data.string.as_bytes()[self.range()]
    }
}

impl Slice<Range<usize>> for SpanData {
    fn slice(&self, range: Range<usize>) -> Self {
        let mut result = self.clone();
        result.start += range.start;
        result.end = result.start + range.len();
        result
    }
}

impl Slice<RangeFrom<usize>> for SpanData {
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        let mut result = self.clone();
        result.start += range.start;
        result
    }
}

impl Slice<RangeTo<usize>> for SpanData {
    fn slice(&self, range: RangeTo<usize>) -> Self {
        let mut result = self.clone();
        result.end = result.start + range.end;
        result
    }
}

impl Slice<RangeFull> for SpanData {
    fn slice(&self, _: RangeFull) -> Self {
        self.clone()
    }
}

impl Offset for SpanData {
    fn offset(&self, second: &Self) -> usize {
        let fst = self.offset();
        let snd = second.offset();
        match fst >= snd {
            true => fst - snd,
            false => snd - fst,
        }
    }
}

impl InputLength for SpanData {
    fn input_len(&self) -> usize {
        self.len()
    }
}

impl FindSubstring<&str> for SpanData {
    fn find_substring(&self, substr: &str) -> Option<usize> {
        self.as_str().find_substring(substr)
    }
}

impl Compare<&str> for SpanData {
    fn compare(&self, t: &str) -> CompareResult {
        self.as_str().compare(t)
    }

    fn compare_no_case(&self, t: &str) -> CompareResult {
        self.as_str().compare_no_case(t)
    }
}

impl Deref for SpanData {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

pub(crate) type SpanInput = LocatedSpan<SpanData>;

pub(crate) fn input_with_path(start: Cow<'static, str>, path: Cow<'static, Path>) -> SpanInput {
    SpanInput::new(SpanData::build(start, Some(path)))
}

pub(crate) fn input_only(i: impl Into<Cow<'static, str>>) -> SpanInput {
    SpanInput::new(SpanData::build(i.into(), None))
}
