use crate::*;

#[derive(Debug, ParseAsInline, ListHashes, Topological, PartialEq)]
pub struct NtValue {
    pub items: Vec<Arc<InlineValue>>,
    pub schema: Arc<InlineSchema>,
}

impl ToOutput for NtValue {
    fn to_output(&self, output: &mut impl Output) {
        for item in &self.items {
            OptionValue::Some(item.clone()).to_output(output);
        }
        self.schema.none_output(output);
    }
}

impl InlineOutput for NtValue {}
impl Tagged for NtValue {}

impl<I: PointInput<Extra = Arc<InlineSchema>>> ParseInline<I> for NtValue
where
    OptionValue<InlineValue>: ParseInline<I>,
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let mut items = Vec::new();
        let schema = loop {
            match input.parse_inline()? {
                OptionValue::Some(item) => items.push(item),
                OptionValue::None(schema) => break schema,
            }
        };
        Ok(Self { items, schema })
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
        self.items.clone()
    }
}
