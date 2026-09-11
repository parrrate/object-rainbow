use crate::*;

#[pod(no_niche)]
pub struct NicheCut;

impl MaybeHasNiche for NicheCut {
    type MnArray = NoNiche<NicheForUnsized>;
}

impl Monostate for NicheCut {}
