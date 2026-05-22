use crate::map_extra::{SmExtra, StaticMap};

pub struct StaticAsciiSplit;

impl<S: AsRef<str>> StaticMap<S> for StaticAsciiSplit {
    type Mapped = Vec<String>;

    fn static_map(x: S) -> Self::Mapped {
        x.as_ref()
            .split_ascii_whitespace()
            .map(From::from)
            .collect()
    }
}

pub type AsciiSplit = SmExtra<StaticAsciiSplit>;
