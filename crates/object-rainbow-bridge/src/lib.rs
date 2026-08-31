use object_rainbow::{Address, Hash};

/// Commands coming from a consumer.
pub enum Consume {
    Inc(Hash),
    Dec(Hash),
    IncChild { parent: Hash, child: Address },
}

/// Responses coming from a provider.
pub enum Provide {
    /// Push a reference towards the client and increase server-side refcount by 1.
    Publish(Hash),
}
