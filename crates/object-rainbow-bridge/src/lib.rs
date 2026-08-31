use object_rainbow::Hash;

/// Commands coming from a consumer.
pub enum Consume {}

/// Responses coming from a provider.
pub enum Provide {
    Publish(Hash),
}
