pub trait Monostate: Default + Eq {
    fn consume(self);
}
