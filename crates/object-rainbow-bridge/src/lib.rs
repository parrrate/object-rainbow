use object_rainbow::{Address, Hash};

/// Commands coming from a consumer.
pub enum Consume {
    /// Request [`Provide::Deliver`]y.
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
    Deliver(Vec<u8>),
    /// Push a reference towards the client and increase server-side refcount by 1.
    Publish(Hash),
}
