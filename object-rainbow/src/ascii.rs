use crate::{
    map_extra::{SmExtra, StaticMap},
    zero_terminated::Zt,
};

pub struct StaticAsciiSplit;

impl<S: AsRef<str>> StaticMap<S> for StaticAsciiSplit {
    type Mapped = Vec<Zt<String>>;

    fn static_map(x: S) -> Self::Mapped {
        x.as_ref()
            .split(['\0', '\t', '\n', '\x0C', '\r', ' '].as_slice())
            .filter(|s| !s.is_empty())
            .map(|x| Zt::new(String::from(x)).expect("no zeroes allowed here"))
            .collect()
    }
}

pub type AsciiSplit = SmExtra<StaticAsciiSplit>;

#[test]
fn ascii_split1() -> crate::Result<()> {
    use crate::{
        map_extra::Compose,
        tuple_extra::{Map1, OneCrossN},
    };
    type AsciiSplit1 = Compose<Map1<AsciiSplit>, OneCrossN>;
    assert_eq!(
        AsciiSplit1::static_map((1, "a b")),
        [(1, "a".parse()?), (1, "b".parse()?)],
    );
    Ok(())
}
