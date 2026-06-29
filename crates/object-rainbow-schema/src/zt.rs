use crate::*;

#[derive(Debug, ToOutput, InlineOutput, ListHashes, Topological, Parse, ParseInline, PartialEq)]
pub struct ZtValue {
    pub schema: Extras<Arc<TailSchema>>,
    pub value: Zt<Arc<TailValue>>,
}
