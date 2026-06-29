use crate::*;

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Parse,
    ParseInline,
    MaybeHasNiche,
    ListHashes,
    Topological,
    Tagged,
    Clone,
    PartialEq,
)]
pub struct ArraySchema {
    pub len: u64,
    pub schema: Arc<InlineSchema>,
}

impl AbstractSchema for ArraySchema {
    fn niche(&self) -> SchemaNiche {
        self.schema.niche().repeat(self.len)
    }
}
