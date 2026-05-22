pub trait InjectAt: Sized + PartialEq {
    fn inject_at() -> Self;
}

pub enum InjectOption<T, O> {
    Injected(Option<O>),
    Proper(T),
}
