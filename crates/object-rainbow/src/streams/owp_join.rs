use std::{
    cmp::Ordering,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;

use super::owp::{Owp, OwpBounds};

#[derive(Default)]
enum State<K, Lv, Rv> {
    #[default]
    Empty,
    L(K, Lv),
    R(K, Rv),
}

impl<K, Lv, Rv> State<K, Lv, Rv> {
    fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

enum WhereNext {
    Owp,
    Min,
    Max,
}

impl WhereNext {
    fn is_waiting(&self) -> bool {
        matches!(self, Self::Owp)
    }

    fn is_min(&self) -> bool {
        matches!(self, Self::Min)
    }

    fn is_max(&self) -> bool {
        matches!(self, Self::Max)
    }
}

#[pin_project]
pub struct OwpJoin<L, R, K, Lv, Rv> {
    #[pin]
    owp_l: Option<L>,
    #[pin]
    owp_r: Option<R>,
    min: State<K, Lv, Rv>,
    max: State<K, Lv, Rv>,
}

impl<L: Owp<Item = (K, Lv)> + OwpBounds<K>, R: Owp<Item = (K, Rv)> + OwpBounds<K>, K: Ord, Lv, Rv>
    OwpJoin<L, R, K, Lv, Rv>
{
    pub fn new(l: L, r: R) -> Self {
        Self {
            owp_l: Some(l),
            owp_r: Some(r),
            min: Default::default(),
            max: Default::default(),
        }
    }
}

impl<L: Owp<Item = (K, Lv)> + OwpBounds<K>, R: Owp<Item = (K, Rv)> + OwpBounds<K>, K: Ord, Lv, Rv>
    Owp for OwpJoin<L, R, K, Lv, Rv>
{
    type Item = (K, (Lv, Rv));

    fn poll_min(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<Self::Item>>> {
        let mut this = self.project();
        loop {
            let next_l_in = if matches!(this.min, State::L(_, _)) {
                WhereNext::Min
            } else if this.owp_l.is_some() {
                WhereNext::Owp
            } else if matches!(this.max, State::L(_, _)) {
                WhereNext::Max
            } else {
                break Poll::Ready(Ok(None));
            };
            let next_r_in = if matches!(this.min, State::R(_, _)) {
                WhereNext::Min
            } else if this.owp_r.is_some() {
                WhereNext::Owp
            } else if matches!(this.max, State::R(_, _)) {
                WhereNext::Max
            } else {
                break Poll::Ready(Ok(None));
            };
            let waiting_l = next_l_in.is_waiting();
            let waiting_r = next_r_in.is_waiting();
            let any_waiting = waiting_l || waiting_r;
            if waiting_l
                && let Poll::Ready(item) = this.owp_l.as_mut().as_pin_mut().unwrap().poll_min(cx)?
            {
                let Some((k_l, v_l)) = item else {
                    this.owp_l.set(None);
                    continue;
                };
                if next_r_in.is_min() {
                    let State::R(k_r, v_r) = std::mem::take(this.min) else {
                        unreachable!()
                    };
                    match k_l.cmp(&k_r) {
                        Ordering::Less => {
                            *this.min = State::R(k_r, v_r);
                        }
                        Ordering::Equal => break Poll::Ready(Ok(Some((k_l, (v_l, v_r))))),
                        Ordering::Greater => {
                            *this.min = State::L(k_l, v_l);
                        }
                    }
                } else {
                    assert!(this.min.is_empty());
                    *this.min = State::L(k_l, v_l);
                }
                continue;
            }
            if waiting_r
                && let Poll::Ready(item) = this.owp_r.as_mut().as_pin_mut().unwrap().poll_min(cx)?
            {
                let Some((k_r, v_r)) = item else {
                    this.owp_r.set(None);
                    continue;
                };
                if next_l_in.is_min() {
                    let State::L(k_l, v_l) = std::mem::take(this.min) else {
                        unreachable!()
                    };
                    match k_l.cmp(&k_r) {
                        Ordering::Less => {
                            *this.min = State::R(k_r, v_r);
                        }
                        Ordering::Equal => break Poll::Ready(Ok(Some((k_l, (v_l, v_r))))),
                        Ordering::Greater => {
                            *this.min = State::L(k_l, v_l);
                        }
                    }
                } else {
                    assert!(this.min.is_empty());
                    *this.min = State::R(k_r, v_r);
                }
                continue;
            };
            if any_waiting {
                break Poll::Pending;
            }
            match (next_l_in, next_r_in) {
                (WhereNext::Min, WhereNext::Max) => {
                    let State::L(k_l, v_l) = std::mem::take(this.min) else {
                        unreachable!()
                    };
                    let State::R(k_r, v_r) = std::mem::take(this.max) else {
                        unreachable!()
                    };
                    match k_l.cmp(&k_r) {
                        Ordering::Less => {
                            *this.max = State::R(k_r, v_r);
                        }
                        Ordering::Equal => break Poll::Ready(Ok(Some((k_l, (v_l, v_r))))),
                        Ordering::Greater => break Poll::Ready(Ok(None)),
                    }
                }
                (WhereNext::Max, WhereNext::Min) => {
                    let State::L(k_l, v_l) = std::mem::take(this.max) else {
                        unreachable!()
                    };
                    let State::R(k_r, v_r) = std::mem::take(this.min) else {
                        unreachable!()
                    };
                    match k_l.cmp(&k_r) {
                        Ordering::Less => break Poll::Ready(Ok(None)),
                        Ordering::Equal => break Poll::Ready(Ok(Some((k_l, (v_l, v_r))))),
                        Ordering::Greater => {
                            *this.max = State::L(k_l, v_l);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    fn poll_max(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<Self::Item>>> {
        let mut this = self.project();
        loop {
            let next_l_in = if matches!(this.max, State::L(_, _)) {
                WhereNext::Max
            } else if this.owp_l.is_some() {
                WhereNext::Owp
            } else if matches!(this.min, State::L(_, _)) {
                WhereNext::Min
            } else {
                break Poll::Ready(Ok(None));
            };
            let next_r_in = if matches!(this.max, State::R(_, _)) {
                WhereNext::Max
            } else if this.owp_r.is_some() {
                WhereNext::Owp
            } else if matches!(this.min, State::R(_, _)) {
                WhereNext::Min
            } else {
                break Poll::Ready(Ok(None));
            };
            let waiting_l = next_l_in.is_waiting();
            let waiting_r = next_r_in.is_waiting();
            let any_waiting = waiting_l || waiting_r;
            if waiting_l
                && let Poll::Ready(item) = this.owp_l.as_mut().as_pin_mut().unwrap().poll_min(cx)?
            {
                let Some((k_l, v_l)) = item else {
                    this.owp_l.set(None);
                    continue;
                };
                if next_r_in.is_max() {
                    let State::R(k_r, v_r) = std::mem::take(this.max) else {
                        unreachable!()
                    };
                    match k_l.cmp(&k_r) {
                        Ordering::Less => {
                            *this.max = State::L(k_l, v_l);
                        }
                        Ordering::Equal => break Poll::Ready(Ok(Some((k_l, (v_l, v_r))))),
                        Ordering::Greater => {
                            *this.max = State::R(k_r, v_r);
                        }
                    }
                } else {
                    assert!(this.max.is_empty());
                    *this.max = State::L(k_l, v_l);
                }
                continue;
            }
            if waiting_r
                && let Poll::Ready(item) = this.owp_r.as_mut().as_pin_mut().unwrap().poll_min(cx)?
            {
                let Some((k_r, v_r)) = item else {
                    this.owp_r.set(None);
                    continue;
                };
                if next_l_in.is_max() {
                    let State::L(k_l, v_l) = std::mem::take(this.max) else {
                        unreachable!()
                    };
                    match k_l.cmp(&k_r) {
                        Ordering::Less => {
                            *this.max = State::L(k_l, v_l);
                        }
                        Ordering::Equal => break Poll::Ready(Ok(Some((k_l, (v_l, v_r))))),
                        Ordering::Greater => {
                            *this.max = State::R(k_r, v_r);
                        }
                    }
                } else {
                    assert!(this.max.is_empty());
                    *this.max = State::R(k_r, v_r);
                }
                continue;
            };
            if any_waiting {
                break Poll::Pending;
            }
            match (next_l_in, next_r_in) {
                (WhereNext::Min, WhereNext::Max) => {
                    let State::L(k_l, v_l) = std::mem::take(this.min) else {
                        unreachable!()
                    };
                    let State::R(k_r, v_r) = std::mem::take(this.max) else {
                        unreachable!()
                    };
                    match k_l.cmp(&k_r) {
                        Ordering::Less => {
                            *this.min = State::L(k_l, v_l);
                        }
                        Ordering::Equal => break Poll::Ready(Ok(Some((k_l, (v_l, v_r))))),
                        Ordering::Greater => break Poll::Ready(Ok(None)),
                    }
                }
                (WhereNext::Max, WhereNext::Min) => {
                    let State::L(k_l, v_l) = std::mem::take(this.max) else {
                        unreachable!()
                    };
                    let State::R(k_r, v_r) = std::mem::take(this.min) else {
                        unreachable!()
                    };
                    match k_l.cmp(&k_r) {
                        Ordering::Less => break Poll::Ready(Ok(None)),
                        Ordering::Equal => break Poll::Ready(Ok(Some((k_l, (v_l, v_r))))),
                        Ordering::Greater => {
                            *this.min = State::R(k_r, v_r);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
