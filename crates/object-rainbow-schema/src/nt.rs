use crate::*;

#[derive(Debug, ParseAsInline, ListHashes, Topological, PartialEq)]
pub struct NtValue {
    pub schema: Arc<InlineSchema>,
    pub items: Vec<Shared<InlineValue>>,
}

impl ToOutput for NtValue {
    fn to_output(&self, output: &mut impl Output) {
        for item in &self.items {
            item.some_output(output);
        }
        self.schema.none_output(output);
    }
}

impl InlineOutput for NtValue {}
impl Tagged for NtValue {}

impl<I: PointInput<Extra = Arc<InlineSchema>>> ParseInline<I> for NtValue
where
    Option<Shared<InlineValue>>: ParseInline<I>,
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let mut items = Vec::new();
        let schema = input.extra().clone();
        while let Some(item) = input.parse_inline()? {
            items.push(item);
        }
        Ok(Self { schema, items })
    }
}

impl AbstractValue for NtValue {
    type Schema = InlineSchema;

    fn schema(&self) -> Self::Schema {
        InlineSchema::Nt(self.schema.clone())
    }
}

impl AbstractCollection for NtValue {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        self.items
            .iter()
            .cloned()
            .map(|Shared(value)| value)
            .collect()
    }
}
