use crate::map_extra::StaticMap;

pub struct StaticAsciiSplit;

impl<S: AsRef<str>> StaticMap<S> for StaticAsciiSplit {
    type Mapped = Vec<String>;

    fn map_extra(x: S) -> Self::Mapped {
        x.as_ref()
            .split_ascii_whitespace()
            .map(From::from)
            .collect()
    }
}
