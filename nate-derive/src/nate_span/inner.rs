use std::ops::{Range, RangeFrom, RangeFull, RangeTo, RangeToInclusive};

use nom::{AsBytes, Compare, CompareResult, FindSubstring, FindToken, InputLength, Offset, Slice};

use super::Shared;

// Should be `pub(crate)`, but in stable (1.58) you get "crate-private type `nate_span::inner::Current<T, S>`
// in public interface". Fixed in nightly (1.60).
#[derive(Debug)]
#[allow(unreachable_pub)]
pub struct Current<T, S>
where
    T: AsRef<str>,
{
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) shared: Shared<T, S>,
}

impl<T, S> Clone for Current<T, S>
where
    T: AsRef<str>,
{
    fn clone(&self) -> Self {
        Self {
            start: self.start,
            end: self.end,
            shared: self.shared.clone(),
        }
    }
}

impl<T, S> Current<T, S>
where
    T: AsRef<str>,
{
    pub(crate) fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    fn offset(&self) -> usize {
        self.start
    }

    fn range(&self) -> Range<usize> {
        self.start..self.start + self.len()
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.shared.0.as_ref()[self.range()]
    }
}

impl<T, S> PartialEq for Current<T, S>
where
    T: AsRef<str>,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<T, S> AsBytes for Current<T, S>
where
    T: AsRef<str>,
{
    fn as_bytes(&self) -> &[u8] {
        &self.shared.0.as_ref().as_bytes()[self.range()]
    }
}

impl<T, S> Slice<Range<usize>> for Current<T, S>
where
    T: AsRef<str>,
{
    fn slice(&self, range: Range<usize>) -> Self {
        let mut result = self.clone();
        result.start += range.start;
        result.end = result.start + range.len();
        result
    }
}

impl<T, S> Slice<RangeFrom<usize>> for Current<T, S>
where
    T: AsRef<str>,
{
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        let mut result = Self::clone(self);
        result.start += range.start;
        result
    }
}

impl<T, S> Slice<RangeTo<usize>> for Current<T, S>
where
    T: AsRef<str>,
{
    fn slice(&self, range: RangeTo<usize>) -> Self {
        let mut result = self.clone();
        result.end = result.start + range.end;
        result
    }
}

impl<T, S> Slice<RangeToInclusive<usize>> for Current<T, S>
where
    T: AsRef<str>,
{
    fn slice(&self, range: RangeToInclusive<usize>) -> Self {
        let mut result = self.clone();
        result.end = result.start + range.end + 1;
        result
    }
}

impl<T, S> Slice<RangeFull> for Current<T, S>
where
    T: AsRef<str>,
{
    fn slice(&self, _: RangeFull) -> Self {
        self.clone()
    }
}

impl<T, S> Offset for Current<T, S>
where
    T: AsRef<str>,
{
    fn offset(&self, second: &Self) -> usize {
        let fst = self.offset();
        let snd = second.offset();
        if fst >= snd { fst - snd } else { snd - fst }
    }
}

impl<T, S> InputLength for Current<T, S>
where
    T: AsRef<str>,
{
    fn input_len(&self) -> usize {
        self.len()
    }
}

#[allow(single_use_lifetimes)]
impl<B, T, S> FindSubstring<B> for Current<T, S>
where
    T: AsRef<str>,
    for<'a> &'a [u8]: FindSubstring<B>,
{
    fn find_substring(&self, substr: B) -> Option<usize> {
        self.as_bytes().find_substring(substr)
    }
}

#[allow(single_use_lifetimes)]
impl<B, T, S> Compare<B> for Current<T, S>
where
    T: AsRef<str>,
    for<'a> &'a [u8]: Compare<B>,
{
    fn compare(&self, t: B) -> CompareResult {
        self.as_bytes().compare(t)
    }

    fn compare_no_case(&self, t: B) -> CompareResult {
        self.as_bytes().compare_no_case(t)
    }
}

#[allow(single_use_lifetimes)]
impl<U, T, S> FindToken<U> for Current<T, S>
where
    T: AsRef<str>,
    for<'a> &'a [u8]: FindToken<U>,
{
    fn find_token(&self, token: U) -> bool {
        self.as_bytes().find_token(token)
    }
}

pub(crate) type LocatedSpan<T, X = (), S = ()> = nom_locate::LocatedSpan<Current<T, S>, X>;
