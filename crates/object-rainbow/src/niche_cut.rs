use crate::*;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    ToOutput,
    InlineOutput,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Size,
    Default,
)]
pub struct NicheCut;

impl ByteOrd for NicheCut {
    fn bytes_cmp(&self, Self: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl MaybeHasNiche for NicheCut {
    type MnArray = NoNiche<NicheForUnsized>;
}

impl Monostate for NicheCut {}
