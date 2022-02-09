#![allow(unused)]
#![allow(unreachable_pub)]

mod inner;
mod outer;

use std::borrow::Cow;
use std::rc::Rc;

pub use outer::SpanAny;

type Shared<T, S> = Rc<(T, S)>;

/// TODO
pub type SpanString<X = (), S = ()> = SpanAny<String, X, S>;

/// TODO
pub type SpanCow<'a, X = (), S = ()> = SpanAny<Cow<'a, str>, X, S>;

/// TODO
pub type SpanStatic<X = (), S = ()> = SpanAny<Cow<'static, str>, X, S>;
