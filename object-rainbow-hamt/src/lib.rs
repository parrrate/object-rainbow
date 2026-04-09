use std::pin::Pin;

use object_rainbow::{
    Enum, Fetch, Hash, Inline, InlineOutput, ListHashes, MaybeHasNiche, Object, Output, Parse,
    ParseInline, PointInput, PointVisitor, Size, SizeExt, Tagged, Tags, ToOutput, Topological,
    Traversible, assert_impl,
    inline_extra::InlineExtra,
    map_extra::{MapExtra, MappedExtra},
    parse_extra::ParseExtra,
    without_header::WithoutHeader,
};
use object_rainbow_array_map::KeyedArrayMap;
use object_rainbow_point::{IntoPoint, Point};

type OptionFuture<'a, T> =
    Pin<Box<dyn 'a + Send + Future<Output = object_rainbow::Result<Option<T>>>>>;

trait Amt<K> {
    type H: Send + Sync;
    type V: Send + Sync;
    fn insert(&mut self, key: K, hash: Self::H, value: Self::V) -> OptionFuture<'_, Self::V>;
    fn from_pair(a: (K, Self::H, Self::V), b: (K, Self::H, Self::V)) -> Self;
    fn get(&self, key: K) -> OptionFuture<'_, (Self::H, Self::V)>;
}

type P1 = u8;
type P2 = (P1, u8);
type P3 = (P2, u8);
type P4 = (P3, u8);
type P5 = (P4, u8);
type P6 = (P5, u8);
type P7 = (P6, u8);
type P8 = (P7, u8);
type P9 = (P8, u8);
type P10 = (P9, u8);
type P11 = (P10, u8);
type P12 = (P11, u8);
type P13 = (P12, u8);
type P14 = (P13, u8);
type P15 = (P14, u8);
type P16 = (P15, u8);
type P17 = (P16, u8);
type P18 = (P17, u8);
type P19 = (P18, u8);
type P20 = (P19, u8);
type P21 = (P20, u8);
type P22 = (P21, u8);
type P23 = (P22, u8);
type P24 = (P23, u8);
type P25 = (P24, u8);
type P26 = (P25, u8);
type P27 = (P26, u8);
type P28 = (P27, u8);
type P29 = (P28, u8);
type P30 = (P29, u8);
type P31 = (P30, u8);

type X32<E> = E;
type X31<E> = (P1, E);
type X30<E> = (P2, E);
type X29<E> = (P3, E);
type X28<E> = (P4, E);
type X27<E> = (P5, E);
type X26<E> = (P6, E);
type X25<E> = (P7, E);
type X24<E> = (P8, E);
type X23<E> = (P9, E);
type X22<E> = (P10, E);
type X21<E> = (P11, E);
type X20<E> = (P12, E);
type X19<E> = (P13, E);
type X18<E> = (P14, E);
type X17<E> = (P15, E);
type X16<E> = (P16, E);
type X15<E> = (P17, E);
type X14<E> = (P18, E);
type X13<E> = (P19, E);
type X12<E> = (P20, E);
type X11<E> = (P21, E);
type X10<E> = (P22, E);
type X9<E> = (P23, E);
type X8<E> = (P24, E);
type X7<E> = (P25, E);
type X6<E> = (P26, E);
type X5<E> = (P27, E);
type X4<E> = (P28, E);
type X3<E> = (P29, E);
type X2<E> = (P30, E);
type X1<E> = (P31, E);

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R1;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R2;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R3;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R4;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R5;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R6;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R7;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R8;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R9;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R10;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R11;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R12;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R13;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R14;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R15;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R16;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R17;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R18;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R19;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R20;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R21;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R22;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R23;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R24;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R25;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R26;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R27;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R28;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R29;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R30;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R31;
#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct R32;

impl<E: 'static + Clone> MapExtra<(u8, X32<E>)> for R32 {
    type Mapped = X31<E>;

    fn map_extra(&self, (key, extra): (u8, X32<E>)) -> Self::Mapped {
        (key, extra)
    }
}

macro_rules! rearrange {
    ($xouter:ident, $router:ident, $xinner:ident) => {
        impl<E: 'static + Clone> MapExtra<(u8, $xouter<E>)> for $router {
            type Mapped = $xinner<E>;

            fn map_extra(&self, (key, (prefix, extra)): (u8, $xouter<E>)) -> Self::Mapped {
                ((prefix, key), extra)
            }
        }
    };
}

rearrange!(X31, R31, X30);
rearrange!(X30, R30, X29);
rearrange!(X29, R29, X28);
rearrange!(X28, R28, X27);
rearrange!(X27, R27, X26);
rearrange!(X26, R26, X25);
rearrange!(X25, R25, X24);
rearrange!(X24, R24, X23);
rearrange!(X23, R23, X22);
rearrange!(X22, R22, X21);
rearrange!(X21, R21, X20);
rearrange!(X20, R20, X19);
rearrange!(X19, R19, X18);
rearrange!(X18, R18, X17);
rearrange!(X17, R17, X16);
rearrange!(X16, R16, X15);
rearrange!(X15, R15, X14);
rearrange!(X14, R14, X13);
rearrange!(X13, R13, X12);
rearrange!(X12, R12, X11);
rearrange!(X11, R11, X10);
rearrange!(X10, R10, X9);
rearrange!(X9, R9, X8);
rearrange!(X8, R8, X7);
rearrange!(X7, R7, X6);
rearrange!(X6, R6, X5);
rearrange!(X5, R5, X4);
rearrange!(X4, R4, X3);
rearrange!(X3, R3, X2);
rearrange!(X2, R2, X1);

impl<E: 'static + Clone> MapExtra<(u8, X1<E>)> for R1 {
    type Mapped = ([u8; 32], E);
    fn map_extra(&self, (key, (prefix, extra)): (u8, X1<E>)) -> Self::Mapped {
        (From::from((prefix, key).to_array()), extra)
    }
}

#[derive(ToOutput, InlineOutput, Tagged, ListHashes, Topological, Parse, ParseInline, Clone)]
struct Entry<V, H = Hash>(ParseExtra<H>, MappedExtra<V, WithoutHeader>);

impl<V, H> Entry<V, H> {
    fn parts(self) -> (H, V) {
        (self.0.0, self.1.1)
    }
}

#[derive(ToOutput, Tagged, ListHashes, Topological, Parse, Clone)]
/// what are you even doing at this point
struct DeepestLeaf<V = (), H = Hash>(KeyedArrayMap<MappedExtra<Entry<V, H>, R1>>);

impl<V: Send + Sync + Clone, H: Send + Sync + Clone> Amt<u8> for DeepestLeaf<V, H> {
    type H = H;
    type V = V;

    fn insert(&mut self, key: u8, hash: Self::H, value: Self::V) -> OptionFuture<'_, Self::V> {
        Box::pin(async move {
            Ok(self
                .0
                .insert(
                    key,
                    MappedExtra(
                        R1,
                        Entry(ParseExtra(hash), MappedExtra(WithoutHeader, value)),
                    ),
                )
                .map(|value| value.1.1.1))
        })
    }

    fn from_pair(
        (ka, ha, va): (u8, Self::H, Self::V),
        (kb, hb, vb): (u8, Self::H, Self::V),
    ) -> Self {
        Self(KeyedArrayMap(
            [
                (
                    ka,
                    MappedExtra(R1, Entry(ParseExtra(ha), MappedExtra(WithoutHeader, va))),
                ),
                (
                    kb,
                    MappedExtra(R1, Entry(ParseExtra(hb), MappedExtra(WithoutHeader, vb))),
                ),
            ]
            .into(),
        ))
    }

    fn get(&self, key: u8) -> OptionFuture<'_, (Self::H, Self::V)> {
        Box::pin(async move { Ok(self.0.get(key).cloned().map(|value| value.1.parts())) })
    }
}

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
)]
struct Merge;

macro_rules! to_array {
    ($k:ident, $x:ident) => {
        impl<E: 'static + Clone> MapExtra<($k, $x<E>)> for Merge {
            type Mapped = ([u8; 32], E);

            fn map_extra(&self, (suffix, (prefix, extra)): ($k, $x<E>)) -> Self::Mapped {
                (From::from((prefix, suffix).to_array()), extra)
            }
        }
    };
}

to_array!(K1, X1);
to_array!(K2, X2);
to_array!(K3, X3);
to_array!(K4, X4);
to_array!(K5, X5);
to_array!(K6, X6);
to_array!(K7, X7);
to_array!(K8, X8);
to_array!(K9, X9);
to_array!(K10, X10);
to_array!(K11, X11);
to_array!(K12, X12);
to_array!(K13, X13);
to_array!(K14, X14);
to_array!(K15, X15);
to_array!(K16, X16);
to_array!(K17, X17);
to_array!(K18, X18);
to_array!(K19, X19);
to_array!(K20, X20);
to_array!(K21, X21);
to_array!(K22, X22);
to_array!(K23, X23);
to_array!(K24, X24);
to_array!(K25, X25);
to_array!(K26, X26);
to_array!(K27, X27);
to_array!(K28, X28);
to_array!(K29, X29);
to_array!(K30, X30);
to_array!(K31, X31);

#[derive(Enum, ToOutput, InlineOutput, Tagged, ListHashes, Topological, Parse, ParseInline)]
enum SubTree<T, K, V = <T as Amt<K>>::V, H = <T as Amt<K>>::H> {
    Leaf(MappedExtra<MappedExtra<Entry<V, H>, Merge>, InlineExtra<K>>),
    SubTree(Point<T>),
}

impl<T, K: Clone, V: Clone, H: Clone> Clone for SubTree<T, K, V, H> {
    fn clone(&self) -> Self {
        match self {
            Self::Leaf(mapped) => Self::Leaf(mapped.clone()),
            Self::SubTree(point) => Self::SubTree(point.clone()),
        }
    }
}

impl<T: Amt<K, H: Clone, V: Clone> + Clone + Traversible, K: Send + Sync + PartialEq + Clone> Amt<K>
    for SubTree<T, K>
{
    type H = T::H;
    type V = T::V;

    fn insert(&mut self, key: K, hash: Self::H, value: Self::V) -> OptionFuture<'_, Self::V> {
        Box::pin(async move {
            match self {
                SubTree::Leaf(MappedExtra(InlineExtra(xkey), MappedExtra(_, xvalue))) => {
                    Ok(if *xkey != key {
                        *self = Self::from_pair(
                            (xkey.clone(), xvalue.0.0.clone(), xvalue.1.1.clone()),
                            (key, hash, value),
                        );
                        None
                    } else {
                        Some(std::mem::replace(&mut xvalue.1.1, value))
                    })
                }
                SubTree::SubTree(sub) => sub.fetch_mut().await?.insert(key, hash, value).await,
            }
        })
    }

    fn from_pair(a: (K, Self::H, Self::V), b: (K, Self::H, Self::V)) -> Self {
        Self::SubTree(T::from_pair(a, b).point())
    }

    fn get(&self, key: K) -> OptionFuture<'_, (Self::H, Self::V)> {
        Box::pin(async move {
            match self {
                SubTree::Leaf(MappedExtra(InlineExtra(existing), MappedExtra(_, value))) => {
                    Ok((*existing == key).then(|| (value.0.0.clone(), value.1.1.clone())))
                }
                SubTree::SubTree(sub) => sub.fetch().await?.get(key).await,
            }
        })
    }
}

#[derive(ToOutput, Tagged, ListHashes, Topological, Parse)]
struct SetNode<T, K, R, V = <T as Amt<K>>::V, H = <T as Amt<K>>::H>(
    KeyedArrayMap<MappedExtra<SubTree<T, K, V, H>, R>>,
);

impl<T, K: Clone, R: Clone, V: Clone, H: Clone> Clone for SetNode<T, K, R, V, H> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T, K, R, V, H> Default for SetNode<T, K, R, V, H> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<
    T: Amt<K, H: Clone, V: Clone> + Clone + Traversible,
    K: Send + Sync + PartialEq + Clone,
    R: Send + Sync + Clone + Default,
> Amt<(u8, K)> for SetNode<T, K, R>
{
    type H = T::H;
    type V = T::V;

    fn insert(
        &mut self,
        (key, rest): (u8, K),
        hash: Self::H,
        value: Self::V,
    ) -> OptionFuture<'_, Self::V> {
        Box::pin(async move {
            if let Some(sub) = self.0.get_mut(key) {
                sub.insert(rest, hash, value).await
            } else {
                assert!(
                    self.0
                        .insert(
                            key,
                            MappedExtra(
                                R::default(),
                                SubTree::Leaf(MappedExtra(
                                    InlineExtra(rest),
                                    MappedExtra(
                                        Merge,
                                        Entry(ParseExtra(hash), MappedExtra(WithoutHeader, value))
                                    )
                                ))
                            )
                        )
                        .is_none()
                );
                Ok(None)
            }
        })
    }

    fn from_pair(
        ((a, rest_a), hash_a, value_a): ((u8, K), Self::H, Self::V),
        ((b, rest_b), hash_b, value_b): ((u8, K), Self::H, Self::V),
    ) -> Self {
        if a == b {
            Self(KeyedArrayMap(
                [(
                    a,
                    MappedExtra(
                        R::default(),
                        SubTree::from_pair((rest_a, hash_a, value_a), (rest_b, hash_b, value_b)),
                    ),
                )]
                .into(),
            ))
        } else {
            Self(KeyedArrayMap(
                [
                    (
                        a,
                        MappedExtra(
                            R::default(),
                            SubTree::Leaf(MappedExtra(
                                InlineExtra(rest_a),
                                MappedExtra(
                                    Merge,
                                    Entry(ParseExtra(hash_a), MappedExtra(WithoutHeader, value_a)),
                                ),
                            )),
                        ),
                    ),
                    (
                        b,
                        MappedExtra(
                            R::default(),
                            SubTree::Leaf(MappedExtra(
                                InlineExtra(rest_b),
                                MappedExtra(
                                    Merge,
                                    Entry(ParseExtra(hash_b), MappedExtra(WithoutHeader, value_b)),
                                ),
                            )),
                        ),
                    ),
                ]
                .into(),
            ))
        }
    }

    fn get(&self, (key, rest): (u8, K)) -> OptionFuture<'_, (Self::H, Self::V)> {
        Box::pin(async move {
            if let Some(sub) = self.0.get(key) {
                sub.get(rest).await
            } else {
                Ok(None)
            }
        })
    }
}

type K1 = u8;
type K2 = (u8, K1);
type K3 = (u8, K2);
type K4 = (u8, K3);
type K5 = (u8, K4);
type K6 = (u8, K5);
type K7 = (u8, K6);
type K8 = (u8, K7);
type K9 = (u8, K8);
type K10 = (u8, K9);
type K11 = (u8, K10);
type K12 = (u8, K11);
type K13 = (u8, K12);
type K14 = (u8, K13);
type K15 = (u8, K14);
type K16 = (u8, K15);
type K17 = (u8, K16);
type K18 = (u8, K17);
type K19 = (u8, K18);
type K20 = (u8, K19);
type K21 = (u8, K20);
type K22 = (u8, K21);
type K23 = (u8, K22);
type K24 = (u8, K23);
type K25 = (u8, K24);
type K26 = (u8, K25);
type K27 = (u8, K26);
type K28 = (u8, K27);
type K29 = (u8, K28);
type K30 = (u8, K29);
type K31 = (u8, K30);
type K32 = (u8, K31);

mod private {
    use object_rainbow::ParseSliceExtra;

    use super::*;
    type N1<V = (), H = Hash> = DeepestLeaf<V, H>;

    macro_rules! next_node {
        ($prev:ident, $next:ident, $pk:ident, $k:ident, $r:ident, $x:ident) => {
            #[derive(Clone)]
            pub struct $next<V = (), H = Hash>(SetNode<$prev<V, H>, $pk, $r, V, H>);

            impl<V, H> Default for $next<V, H> {
                fn default() -> Self {
                    Self(Default::default())
                }
            }

            impl<
                V: Inline<Extra>,
                H: Object<Extra>,
                I: PointInput<Extra = $x<Extra>>,
                Extra: 'static + Send + Sync + Clone,
            > Parse<I> for $next<V, H>
            {
                fn parse(input: I) -> object_rainbow::Result<Self> {
                    let extra = &input.extra().clone();
                    let resolve = input.resolve();
                    let data = input.parse_all()?;
                    Ok(Self(ParseSliceExtra::parse_slice_extra(
                        &data, &resolve, extra,
                    )?))
                }
            }

            impl<V: Tagged, H: Tagged> Tagged for $next<V, H> {
                const TAGS: Tags = <(V, H) as Tagged>::TAGS;
            }

            impl<V: InlineOutput, H: ToOutput> ToOutput for $next<V, H> {
                fn to_output(&self, output: &mut dyn Output) {
                    self.0.to_output(output);
                }
            }

            impl<V: ListHashes, H: ListHashes> ListHashes for $next<V, H> {
                fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
                    self.0.list_hashes(f)
                }
            }

            impl<V: Traversible + InlineOutput, H: Traversible> Topological for $next<V, H> {
                fn traverse(&self, visitor: &mut impl PointVisitor) {
                    self.0.traverse(visitor)
                }
            }

            impl<V: Traversible + InlineOutput + Clone, H: Traversible + Clone> Amt<$k>
                for $next<V, H>
            {
                type H = H;
                type V = V;

                fn insert(
                    &mut self,
                    key: $k,
                    hash: Self::H,
                    value: Self::V,
                ) -> OptionFuture<'_, Self::V> {
                    self.0.insert(key, hash, value)
                }

                fn from_pair(a: ($k, Self::H, Self::V), b: ($k, Self::H, Self::V)) -> Self {
                    Self(Amt::from_pair(a, b))
                }

                fn get(&self, key: $k) -> OptionFuture<'_, (Self::H, Self::V)> {
                    self.0.get(key)
                }
            }
        };
    }

    next_node!(N1, N2, K1, K2, R2, X2);
    next_node!(N2, N3, K2, K3, R3, X3);
    next_node!(N3, N4, K3, K4, R4, X4);
    next_node!(N4, N5, K4, K5, R5, X5);
    next_node!(N5, N6, K5, K6, R6, X6);
    next_node!(N6, N7, K6, K7, R7, X7);
    next_node!(N7, N8, K7, K8, R8, X8);
    next_node!(N8, N9, K8, K9, R9, X9);
    next_node!(N9, N10, K9, K10, R10, X10);
    next_node!(N10, N11, K10, K11, R11, X11);
    next_node!(N11, N12, K11, K12, R12, X12);
    next_node!(N12, N13, K12, K13, R13, X13);
    next_node!(N13, N14, K13, K14, R14, X14);
    next_node!(N14, N15, K14, K15, R15, X15);
    next_node!(N15, N16, K15, K16, R16, X16);
    next_node!(N16, N17, K16, K17, R17, X17);
    next_node!(N17, N18, K17, K18, R18, X18);
    next_node!(N18, N19, K18, K19, R19, X19);
    next_node!(N19, N20, K19, K20, R20, X20);
    next_node!(N20, N21, K20, K21, R21, X21);
    next_node!(N21, N22, K21, K22, R22, X22);
    next_node!(N22, N23, K22, K23, R23, X23);
    next_node!(N23, N24, K23, K24, R24, X24);
    next_node!(N24, N25, K24, K25, R25, X25);
    next_node!(N25, N26, K25, K26, R26, X26);
    next_node!(N26, N27, K26, K27, R27, X27);
    next_node!(N27, N28, K27, K28, R28, X28);
    next_node!(N28, N29, K28, K29, R29, X29);
    next_node!(N29, N30, K29, K30, R30, X30);
    next_node!(N30, N31, K30, K31, R31, X31);
    next_node!(N31, N32, K31, K32, R32, X32);
}

#[derive(
    ToOutput, InlineOutput, Tagged, ListHashes, Topological, Parse, ParseInline, Size, MaybeHasNiche,
)]
pub struct HamtMap<V, H = Hash>(Point<private::N32<V, H>>);

assert_impl!(
    impl<V, H, E> Inline<E> for HamtMap<V, H>
    where
        E: 'static + Send + Sync + Clone,
        V: Inline<E>,
        H: Object<E>,
    {
    }
);

impl<V, H> Clone for HamtMap<V, H> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<V, H> PartialEq for HamtMap<V, H> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<V, H> Eq for HamtMap<V, H> {}

impl<V: Traversible + InlineOutput + Clone, H: Traversible + Clone> Default for HamtMap<V, H> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<
    V: Traversible + InlineOutput + Clone,
    H: Traversible + Clone + Size<Size = <Hash as Size>::Size>,
> HamtMap<V, H>
{
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn insert(&mut self, hash: H, value: V) -> object_rainbow::Result<Option<V>> {
        self.0
            .fetch_mut()
            .await?
            .insert(hash_key(From::from(hash.to_array())), hash, value)
            .await
    }

    pub async fn get(&self, hash: Hash) -> object_rainbow::Result<Option<V>> {
        self.0
            .fetch()
            .await?
            .get(hash_key(hash.into_bytes()))
            .await
            .map(|value| value.map(|(_, value)| value))
    }
}

fn hash_key(hash: [u8; 32]) -> K32 {
    let [
        x0,
        x1,
        x2,
        x3,
        x4,
        x5,
        x6,
        x7,
        x8,
        x9,
        x10,
        x11,
        x12,
        x13,
        x14,
        x15,
        x16,
        x17,
        x18,
        x19,
        x20,
        x21,
        x22,
        x23,
        x24,
        x25,
        x26,
        x27,
        x28,
        x29,
        x30,
        x31,
    ] = hash;
    (x0,(x1,(x2,(x3,(x4,(x5,(x6,(x7,(x8,(x9,(x10,(x11,(x12,(x13,(x14,(x15,(x16,(x17,(x18,(x19,(x20,(x21,(x22,(x23,(x24,(x25,(x26,(x27,(x28,(x29,(x30,(x31))))))))))))))))))))))))))))))))
}

#[derive(
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Size,
    MaybeHasNiche,
    Clone,
    Default,
    PartialEq,
    Eq,
)]
pub struct HamtSet(HamtMap<()>);

assert_impl!(
    impl<E> Inline<E> for HamtSet where E: 'static + Send + Sync + Clone {}
);

impl HamtSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn insert(&mut self, hash: Hash) -> object_rainbow::Result<bool> {
        Ok(self.0.insert(hash, ()).await?.is_none())
    }

    pub async fn contains(&self, hash: Hash) -> object_rainbow::Result<bool> {
        Ok(self.0.get(hash).await?.is_some())
    }
}
