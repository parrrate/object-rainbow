use crate::*;

#[derive(Debug, ParseAsInline, ListHashes, Topological, PartialEq)]
pub struct NtValue {
    pub items: Vec<Arc<InlineValue>>,
    pub schema: Arc<InlineSchema>,
}

impl ToOutput for NtValue {
    fn to_output(&self, output: &mut impl Output) {
        for item in &self.items {
            ValueOption::Some(item.clone()).to_output(output);
        }
        ValueOption::<InlineValue>::None(self.schema.clone()).to_output(output);
    }
}

impl InlineOutput for NtValue {}
