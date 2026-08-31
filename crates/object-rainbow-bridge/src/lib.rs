use object_rainbow::Hash;

/// Commands coming from a consumer.
pub enum Consume {
    Inc(Hash),
    Dec(Hash),
}

/// Responses coming from a provider.
pub enum Provide {
    /// Push a reference towards the client and increase server-side refcount by 1.
    Publish(Hash),
}
