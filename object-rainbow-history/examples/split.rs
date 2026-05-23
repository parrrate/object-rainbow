use macro_rules_attribute::apply;
use object_rainbow::{
    InlineOutput, ListHashes, Parse, ParseInline, Tagged, ToOutput, Topological,
    ascii::AsciiSplit1,
    length_prefixed::LpString,
    map_extra::{Compose, FMap, Return, UniqueSorted},
    tuple_extra::{Map1, OneCrossN, Swap},
};
use object_rainbow_amt::{AmtMap, AmtSet};
use object_rainbow_history::{
    Apply, FromIter, Parallel, Sequential,
    remap::{MapToSet, MappedToSet},
};
use smol_macros::main;
use ulid::Ulid;

#[derive(
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
)]
struct ToSet;

impl MapToSet<Ulid, LpString> for ToSet {
    type T = (Ulid, LpString);

    fn map(
        &self,
        key: Ulid,
        value: LpString,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::T>> {
        core::future::ready(Ok((key, value)))
    }
}

type History = Sequential<
    Parallel<AmtMap<Ulid, LpString>, Return>,
    Sequential<
        MappedToSet<ToSet>,
        FromIter<
            Sequential<
                Compose<Compose<Map1<Compose<AsciiSplit1, FMap<Swap>>>, OneCrossN>, UniqueSorted>,
                FromIter<AmtSet<(LpString, Ulid)>>,
            >,
        >,
    >,
>;

#[apply(main!)]
async fn main() -> object_rainbow::Result<()> {
    let mut history = History::default();
    let id = Ulid::new();
    let x = history.apply((Some("a b a".into()), id)).await?;
    println!("{x:?}");
    for key in ["a", "b"] {
        assert!(
            history
                .second()
                .second()
                .0
                .second()
                .0
                .contains(&(key.into(), id))
                .await?,
        );
    }
    Ok(())
}
