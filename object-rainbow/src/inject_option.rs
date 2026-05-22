pub trait InjectAt: Sized {
    fn inject_at() -> Self;
}

pub enum InjectOption<T, O> {
    Injected(Option<O>),
    Proper(T),
}
