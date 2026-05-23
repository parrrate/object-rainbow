use crate::{
    length_prefixed::LpString,
    map_extra::{SmExtra, StaticMap},
};

pub struct StaticAsciiSplit;

impl<S: AsRef<str>> StaticMap<S> for StaticAsciiSplit {
    type Mapped = Vec<LpString>;

    fn static_map(x: S) -> Self::Mapped {
        x.as_ref()
            .split_ascii_whitespace()
            .map(From::from)
            .collect()
    }
}

pub type AsciiSplit = SmExtra<StaticAsciiSplit>;

#[test]
fn ascii_split1() {
    use crate::{
        map_extra::Compose,
        tuple_extra::{Map1, OneCrossN},
    };
    type AsciiSplit1 = Compose<Map1<AsciiSplit>, OneCrossN>;
    assert_eq!(
        AsciiSplit1::static_map((1, "a b")),
        [(1, "a".into()), (1, "b".into())],
    );
}
