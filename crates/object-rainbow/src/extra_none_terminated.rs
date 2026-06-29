use crate::{extras::Extras, none_terminated::Nt};

#[derive(Debug)]
pub struct Ent<T, E = ()> {
    pub extra: Extras<E>,
    pub items: Nt<T>,
}
