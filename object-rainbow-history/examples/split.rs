use macro_rules_attribute::apply;
use object_rainbow::{
    ascii::AsciiSplit,
    length_prefixed::LpString,
    map_extra::{Compose, FMap, Flatten, Return, UniqueSorted},
    tuple_extra::{Map1, OneCrossN, Swap},
};
use object_rainbow_amt::{AmtMap, AmtSet};
use object_rainbow_history::{
    Apply, FromIter, Parallel, Sequential,
    remap::{MappedToSet, ToSet},
};
use smol_macros::main;
use ulid::Ulid;

type WordSearch = Sequential<
    Sequential<Parallel<AmtMap<Ulid, LpString>, Return>, MappedToSet<ToSet>>,
    Sequential<
        FromIter<
            Sequential<
                Compose<
                    Map1<
                        Compose<
                            Compose<Map1<Compose<AsciiSplit, UniqueSorted>>, OneCrossN>,
                            FMap<Swap>,
                        >,
                    >,
                    OneCrossN,
                >,
                FromIter<Parallel<AmtSet<(LpString, Ulid)>, Return>>,
            >,
        >,
        Flatten,
    >,
>;

#[apply(main!)]
async fn main() -> object_rainbow::Result<()> {
    let mut history = WordSearch::default();
    let id = Ulid::new();
    let x = history.apply((Some("a b a".into()), id)).await?;
    for (a, (b, (c, d))) in x {
        println!("{a} {b} {c} {d}");
    }
    for key in ["a", "b"] {
        assert!(
            history
                .second()
                .first()
                .0
                .second()
                .0
                .second()
                .0
                .contains(&(key.into(), id))
                .await?,
        );
    }
    println!();
    let x = history.apply((Some("a b c".into()), id)).await?;
    for (a, (b, (c, d))) in x {
        println!("{a} {b} {c} {d}");
    }
    Ok(())
}
