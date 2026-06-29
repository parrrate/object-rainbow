pub enum ExtraNone<T, E = ()> {
    Some(T),
    None(E),
}
