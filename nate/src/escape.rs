#[cfg(feature = "std")]
extern crate std;

use core::fmt::{self, Write as _};

use crate::details::EscapeWrapper;

impl<E: fmt::Display> EscapeKind for &EscapeWrapper<E> {}

#[doc(hidden)]
pub trait EscapeKind {
    #[inline]
    fn wrap<'a, T: fmt::Display>(&self, value: &'a T) -> XmlEscape<&'a T> {
        XmlEscape(value)
    }
}

/// A wrapper around a [displayable][fmt::Display] type that makes it write out XML escaped.
///
/// All characters are written as is except `"`, `&`, `'`, `<`, and `>` which are printed as e.g.
/// `&#34;`.
pub struct XmlEscape<T: ?Sized>(pub T);

const _: () = {
    impl<T: ?Sized + fmt::Display> fmt::Display for XmlEscape<T> {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(XmlEscapeWriter(f), "{}", &self.0)
        }
    }

    impl<T: ?Sized + fmt::Debug> fmt::Debug for XmlEscape<T> {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(XmlEscapeWriter(f), "{:?}", &self.0)
        }
    }

    struct XmlEscapeWriter<'a, 'b>(&'a mut fmt::Formatter<'b>);

    const MIN_CHAR: u8 = b'"';
    const MAX_CHAR: u8 = b'>';
    const TABLE: [Option<&&str>; (MAX_CHAR - MIN_CHAR + 1) as usize] = {
        let mut table = [None; (MAX_CHAR - MIN_CHAR + 1) as usize];
        table[(b'"' - MIN_CHAR) as usize] = Some(&"&#34;");
        table[(b'&' - MIN_CHAR) as usize] = Some(&"&#38;");
        table[(b'\'' - MIN_CHAR) as usize] = Some(&"&#39;");
        table[(b'<' - MIN_CHAR) as usize] = Some(&"&#60;");
        table[(b'>' - MIN_CHAR) as usize] = Some(&"&#62;");
        table
    };

    impl fmt::Write for XmlEscapeWriter<'_, '_> {
        fn write_str(&mut self, string: &str) -> fmt::Result {
            let mut last = 0;
            for (index, byte) in string.bytes().enumerate() {
                let escaped = match byte {
                    MIN_CHAR..=MAX_CHAR => TABLE[(byte - MIN_CHAR) as usize],
                    _ => None,
                };
                if let Some(escaped) = escaped {
                    self.0.write_str(&string[last..index])?;
                    self.0.write_str(escaped)?;
                    last = index + 1;
                }
            }
            self.0.write_str(&string[last..])
        }
    }
};
