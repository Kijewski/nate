use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use nom::error::{ErrorKind, ParseError};
use nom::{
    AsBytes, Compare, CompareResult, FindSubstring, FindToken, IResult, InputIter, InputLength,
    InputTake, InputTakeAtPosition, Offset, ParseTo, Slice,
};

use super::inner::{Current, LocatedSpan};
use super::Shared;

/// TODO
#[derive(Clone, Debug)]
pub struct SpanAny<T, X = (), S = ()>(LocatedSpan<T, X, S>)
where
    T: Clone + AsRef<str>,
    X: Clone;

// constructors

impl<T, X, S> SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone + Default,
    S: Default,
{
    /// TODO
    pub fn new(source: impl Into<T>) -> Self {
        Self::new_with_both(source, X::default(), S::default())
    }
}

impl<T, X, S> SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone + Default,
{
    /// TODO
    pub fn new_with_shared(source: impl Into<T>, shared: impl Into<S>) -> Self {
        Self::new_with_both(source, X::default(), shared)
    }
}

impl<T, X, S> SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone,
    S: Default,
{
    /// TODO
    pub fn new_with_extra(source: impl Into<T>, extra: impl Into<X>) -> Self {
        Self::new_with_both(source, extra, S::default())
    }
}

impl<T, X, S> SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone,
{
    /// TODO
    pub fn new_with_both(source: impl Into<T>, extra: impl Into<X>, shared: impl Into<S>) -> Self {
        let source = source.into();
        let extra = extra.into();
        let shared = shared.into();
        let current = Current {
            start: 0,
            end: source.as_ref().len(),
            shared: Shared::new((source, shared)),
        };
        Self(LocatedSpan::new_extra(current, extra))
    }

    //  methods:

    /// Get the string slice for the current span
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Get the original source which the current span is a subspan of
    pub fn get_source(&self) -> &str {
        self.0.shared.0.as_ref()
    }

    /// Get the shared data of this soan
    pub fn get_shared(&self) -> &S {
        &self.0.shared.1
    }

    /// Get the reference counter of the souce and shared data of the span
    pub fn get_rc(&self) -> Shared<T, S> {
        self.0.shared.clone()
    }

    /// Get a reference to the extra data of the span
    pub fn get_extra(&self) -> &X {
        &self.0.extra
    }

    /// Get a mutable reference to the extra data of the span
    pub fn get_extra_mut(&mut self) -> &mut X {
        &mut self.0.extra
    }

    /// Trim the span on both sides
    pub fn trim(&self) -> Self {
        let i = self.as_str();
        let trimmed = i.trim_start();
        let start = i.len() - trimmed.len();
        self.slice(start..start + trimmed.trim_end().len())
    }

    /// Trim the span from its start
    pub fn trim_start(&self) -> Self {
        let i = self.as_str();
        self.slice(i.len() - i.trim_start().len()..)
    }

    /// Trim the span from its end
    pub fn trim_end(&self) -> Self {
        self.slice(..self.as_str().trim_end().len())
    }

    // nom_locate forwards:

    /// Return the column index, assuming 1 byte = 1 column
    ///
    /// See the documentation in [`nom_locate::LocatedSpan::get_column()`].
    pub fn get_column(&self) -> usize {
        self.0.get_column()
    }

    /// Return the column index for UTF-8 text
    ///
    /// See the documentation in [`nom_locate::LocatedSpan::get_utf8_column()`].
    pub fn get_utf8_column(&self) -> usize {
        self.0.get_column()
    }

    /// The line number of the fragment relatively to the input of the parser
    ///
    /// See the documentation in [`nom_locate::LocatedSpan::location_line()`].
    pub fn location_line(&self) -> u32 {
        self.0.location_line()
    }

    /// The offset represents the position of the fragment relatively to the input of the parser
    ///
    /// See the documentation in [`nom_locate::LocatedSpan::location_offset()`].
    pub fn location_offset(&self) -> usize {
        self.0.location_offset()
    }

    /// Return the column index for UTF8 text
    ///
    /// See the documentation in [`nom_locate::LocatedSpan::naive_get_utf8_column()`].
    pub fn naive_get_utf8_column(&self) -> usize {
        self.0.naive_get_utf8_column()
    }
}

// std trait implementations

impl<T, X, S> AsRef<str> for SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone,
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<T, X, S> Deref for SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone,
{
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<T, X, S> fmt::Display for SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<T, X, S> PartialEq for SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.location_offset() == other.location_offset() && self.len() == other.len()
    }
}

// nom trait implementations

impl<T, X, S> AsBytes for SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone,
{
    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl<B, T, X, S> Compare<B> for SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone,
    LocatedSpan<T, X, S>: Compare<B>,
{
    fn compare(&self, t: B) -> CompareResult {
        self.0.compare(t)
    }

    fn compare_no_case(&self, t: B) -> CompareResult {
        self.0.compare_no_case(t)
    }
}

impl<U, T, X, S> FindSubstring<U> for SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone,
    LocatedSpan<T, X, S>: FindSubstring<U>,
{
    fn find_substring(&self, substr: U) -> Option<usize> {
        self.0.find_substring(substr)
    }
}

impl<U, T, X, S> FindToken<U> for SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone,
    LocatedSpan<T, X, S>: FindToken<U>,
{
    fn find_token(&self, token: U) -> bool {
        self.0.find_token(token)
    }
}

impl<T, X, S> InputLength for SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone,
{
    fn input_len(&self) -> usize {
        self.len()
    }
}

impl<T, X, S> InputTake for SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone,
    LocatedSpan<T, X, S>: InputTake,
{
    fn take(&self, count: usize) -> Self {
        Self(self.0.take(count))
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (i, s) = self.0.take_split(count);
        (Self(i), Self(s))
    }
}

impl<T, X, S> InputTakeAtPosition for SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone,
    LocatedSpan<T, X, S>: InputTakeAtPosition + InputTake,
{
    type Item = char;

    fn split_at_position<P, E: ParseError<Self>>(&self, predicate: P) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.0.as_str().position(predicate) {
            None => Err(nom::Err::Incomplete(nom::Needed::new(1))),
            Some(n) => Ok(self.take_split(n)),
        }
    }

    fn split_at_position1<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        e: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.0.as_str().position(predicate) {
            None => Err(nom::Err::Incomplete(nom::Needed::new(1))),
            Some(0) => Err(nom::Err::Error(E::from_error_kind(Self(self.0.clone()), e))),
            Some(n) => Ok(self.take_split(n)),
        }
    }

    fn split_at_position_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.0.as_str().find(predicate) {
            None => Ok(self.take_split(self.len())),
            Some(i) => Ok(self.take_split(i)),
        }
    }

    fn split_at_position1_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        e: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.0.as_str().find(predicate) {
            None => Ok(self.take_split(self.len())),
            Some(0) => Err(nom::Err::Error(E::from_error_kind(Self(self.0.clone()), e))),
            Some(i) => Ok(self.take_split(i)),
        }
    }
}

impl<T, X, S> Offset for SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone,
{
    fn offset(&self, second: &Self) -> usize {
        self.0.offset(&second.0)
    }
}

impl<R, T, X, S> ParseTo<R> for SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone,
    R: FromStr,
{
    fn parse_to(&self) -> Option<R> {
        self.as_str().parse_to()
    }
}

impl<R, T, X: Clone, S> Slice<R> for SpanAny<T, X, S>
where
    T: Clone + AsRef<str>,
    X: Clone,
    LocatedSpan<T, X, S>: Slice<R>,
{
    fn slice(&self, range: R) -> Self {
        Self(self.0.slice(range))
    }
}
