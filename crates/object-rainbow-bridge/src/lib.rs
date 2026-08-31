use object_rainbow::{Address, Hash};

/// Commands coming from a consumer.
pub enum Consume {
    /// Request [`Provide::Deliver`]y. Requires refcount of at least 1.
    Order(Hash),
    /// Increase server-side refcount by 1.
    Inc(Hash),
    /// Decrease server-side refcount by 1.
    Dec(Hash),
    IncChild {
        parent: Hash,
        child: Address,
    },
}

/// Responses coming from a provider.
pub enum Provide {
    /// Fulfil an [`Consume::Order`].
    Deliver(Vec<u8>),
    /// Push a reference towards the client and increase server-side refcount by 1.
    Publish(Hash),
}
