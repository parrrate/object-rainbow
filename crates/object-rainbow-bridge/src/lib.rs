use object_rainbow::{Address, Hash};

/// Commands coming from a consumer.
pub enum Consume {
    Order(Hash),
    Inc(Hash),
    Dec(Hash),
    IncChild { parent: Hash, child: Address },
}

/// Responses coming from a provider.
pub enum Provide {
    Deliver(Vec<u8>),
    /// Push a reference towards the client and increase server-side refcount by 1.
    Publish(Hash),
}
