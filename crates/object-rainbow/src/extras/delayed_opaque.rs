use crate::*;

pub trait ParseDelayedOpaque<E: 'static + Clone>: Sized {
    fn parse_delayed_opaque<I: PointInput<Extra: Fetch<T = E>>>(input: I) -> Result<Self>;
}
