use darling::FromMeta;

/// Whitespace handling of the input source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromMeta)]
pub(crate) enum Strip {
    /// Don't strip any spaces in the input.
    None,
    /// Remove a single single newline at the end of the input. This is the default.
    Tail,
    /// Remove all whitespaces at the front and back all lines, and remove empty lines.
    Trim,
    /// Like Trim, but also replace runs of whitespaces with a single space.
    Eager,
}

impl Default for Strip {
    fn default() -> Self {
        Strip::None
    }
}

impl Strip {
    #[allow(unused)] // TODO
    pub(crate) fn apply(self, mut src: String) -> String {
        match self {
            Strip::None => src,
            Strip::Tail => {
                if src.ends_with('\n') {
                    let _ = src.pop();
                }
                src
            },
            Strip::Trim | Strip::Eager => {
                let mut stripped = String::with_capacity(src.len());
                for line in src.lines().map(|s| s.trim()).filter(|&s| !s.is_empty()) {
                    if !stripped.is_empty() {
                        stripped.push('\n');
                    }
                    if self == Strip::Eager {
                        for (index, word) in line.split_ascii_whitespace().enumerate() {
                            if index > 0 {
                                stripped.push(' ');
                            }
                            stripped.push_str(word);
                        }
                    } else {
                        stripped.push_str(line);
                    }
                }
                stripped
            },
        }
    }
}
