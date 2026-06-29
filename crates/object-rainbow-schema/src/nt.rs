use crate::*;

#[derive(Debug, ParseAsInline, ListHashes, Topological, PartialEq)]
pub struct NtValue {
    pub items: Vec<Arc<InlineValue>>,
    pub schema: Arc<InlineSchema>,
}
