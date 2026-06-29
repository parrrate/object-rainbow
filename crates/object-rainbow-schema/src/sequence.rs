use crate::*;

#[derive(Debug, ToOutput, ListHashes, Topological, Parse, PartialEq)]
pub struct SequenceValue {
    pub schema: Extras<Arc<InlineSchema>>,
    pub items: Vec<Arc<InlineValue>>,
}
