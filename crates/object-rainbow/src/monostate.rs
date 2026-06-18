pub trait Monostate: Default {
    fn consume(self);
}
