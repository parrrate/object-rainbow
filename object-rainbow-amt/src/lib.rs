use object_rainbow::{
    Enum, Fetch, Inline, InlineOutput, ListHashes, ParseAsInline, ParseInline, PointInput, Tagged,
    ToOutput, Traversible,
    enumkind::{EnumKind, EnumParseInline},
    length_prefixed::LpBytes,
    map_extra::MappedExtra,
    without_header::WithoutHeader,
};
use object_rainbow_array_map::KeyedArrayMap;
use object_rainbow_parse_prefix::{Prefix, WithByte, WithBytes, WithPrefix};
use object_rainbow_point::{IntoPoint, Point};

#[derive(Enum, ToOutput, InlineOutput, Tagged, ListHashes, ParseAsInline, Clone, Default)]
#[topology(recursive)]
enum Node<K, V> {
    Leaf(WithPrefix<K>, MappedExtra<V, WithoutHeader>),
    Sub(
        #[tags(skip)]
        #[allow(clippy::type_complexity, reason = "no hope")]
        Point<MappedExtra<KeyedArrayMap<MappedExtra<Self, WithByte>>, WithBytes>>,
    ),
    #[default]
    Empty,
}

impl<K, V> ::object_rainbow::Topological for Node<K, V>
where
    K: InlineOutput + Traversible,
    V: InlineOutput + Traversible,
{
    fn traverse(&self, visitor: &mut impl ::object_rainbow::PointVisitor) {
        let kind = self.kind();
        let tag = kind.to_tag();
        tag.traverse(visitor);
        match self {
            Self::Leaf(k, v) => {
                k.traverse(visitor);
                v.traverse(visitor)
            }
            Self::Sub(point) => point.traverse(visitor),
            Self::Empty => {}
        }
    }
}

impl<
    K: ParseInline<I::WithExtra<E>> + InlineOutput + Traversible + Inline<E>,
    V: ParseInline<I::WithExtra<E>> + InlineOutput + Traversible + Inline<E>,
    I: PointInput<Extra = (Prefix, E)>,
    E: 'static + Send + Sync + Clone,
> ParseInline<I> for Node<K, V>
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        EnumParseInline::parse_as_inline_enum(input)
    }
}

impl<
    K: ParseInline<I::WithExtra<E>> + InlineOutput + Traversible + Inline<E>,
    V: ParseInline<I::WithExtra<E>> + InlineOutput + Traversible + Inline<E>,
    I: PointInput<Extra = (Prefix, E)>,
    E: 'static + Send + Sync + Clone,
> EnumParseInline<I> for Node<K, V>
{
    fn enum_parse_inline(
        kind: <Self as Enum>::Kind,
        input: &mut I,
    ) -> object_rainbow::Result<Self> {
        Ok(match kind {
            <Self as Enum>::Kind::Leaf => Self::Leaf(input.parse_inline()?, input.parse_inline()?),
            <Self as Enum>::Kind::Sub => Self::Sub(input.parse_inline()?),
            <Self as Enum>::Kind::Empty => Self::Empty,
        })
    }
}

impl<K: InlineOutput + Traversible + Clone, V: InlineOutput + Traversible + Clone> Node<K, V> {
    async fn get(&self, key: &[u8]) -> object_rainbow::Result<Option<V>> {
        match self {
            Self::Leaf(k, MappedExtra(_, v)) if k.vec() == key => Ok(Some(v.clone())),
            Self::Sub(point)
                if let MappedExtra(prefix, children) = point.fetch().await?
                    && let Some(key) = key.strip_prefix(&*prefix.0.0)
                    && let Some((first, key)) = key.split_first()
                    && let Some(sub) = children.get(*first) =>
            {
                Box::pin(sub.get(key)).await
            }
            _ => Ok(None),
        }
    }

    fn from_kv(key: &[u8], k: K, v: V) -> object_rainbow::Result<Self> {
        let vec = k.vec();
        let Some(prefix) = vec.strip_suffix(key) else {
            return Err(object_rainbow::error_consistency!("key data mismatch"));
        };
        Ok(Self::Leaf(
            WithPrefix::new(Prefix::from(prefix), k)?,
            MappedExtra(WithoutHeader, v),
        ))
    }

    fn from_pair(
        common: &[u8],
        first_a: u8,
        first_b: u8,
        node_a: Self,
        node_b: Self,
    ) -> object_rainbow::Result<Self> {
        Ok(Self::Sub(
            MappedExtra(
                WithBytes(LpBytes(common.into())),
                KeyedArrayMap(
                    [
                        (first_a, MappedExtra(WithByte, node_a)),
                        (first_b, MappedExtra(WithByte, node_b)),
                    ]
                    .into(),
                ),
            )
            .point(),
        ))
    }

    fn from_kv_pairs(
        prefix: Prefix,
        key_a: &[u8],
        key_b: &[u8],
        k_a: K,
        k_b: K,
        v_a: V,
        v_b: V,
    ) -> object_rainbow::Result<Self> {
        let n = common_length(key_a, key_b)?;
        let common = &key_a[..n];
        let prefix = prefix.with(common);
        let (&first_a, key_a) = key_a[n..].split_first().expect("must have 1 different");
        let (&first_b, key_b) = key_b[n..].split_first().expect("must have 1 different");
        let wp_a = WithPrefix::new(prefix.with(vec![first_a]), k_a)?;
        assert_eq!(wp_a.vec(), key_a);
        let node_a = Self::Leaf(wp_a, MappedExtra(WithoutHeader, v_a));
        let wp_b = WithPrefix::new(prefix.with(vec![first_b]), k_b)?;
        assert_eq!(wp_b.vec(), key_b);
        let node_b = Self::Leaf(wp_b, MappedExtra(WithoutHeader, v_b));
        Self::from_pair(common, first_a, first_b, node_a, node_b)
    }

    async fn insert(
        &mut self,
        key: &[u8],
        k_new: K,
        v_new: V,
    ) -> object_rainbow::Result<Option<V>> {
        match &mut *self {
            Self::Leaf(k, MappedExtra(_, v)) => {
                let vec = k.vec();
                if vec == key {
                    Ok(Some(std::mem::replace(v, v_new)))
                } else {
                    let Self::Leaf(k, MappedExtra(_, v)) = std::mem::take(self) else {
                        unreachable!()
                    };
                    let prefix = k.prefix().clone();
                    *self =
                        Self::from_kv_pairs(prefix, &vec, key, k.into_value(), k_new, v, v_new)?;
                    Ok(None)
                }
            }
            Self::Sub(point) => {
                let (first_a, n) = {
                    let MappedExtra(prefix, children) = &mut *point.fetch_mut().await?;
                    if children.len() < 2 {
                        return Err(object_rainbow::error_consistency!("node too small"));
                    }
                    let key_a = &*prefix.0.0;
                    if let Some(key) = key.strip_prefix(key_a) {
                        let Some((&first, key)) = key.split_first() else {
                            return Err(object_rainbow::error_consistency!(
                                "key is prefix of another key"
                            ));
                        };
                        if !children.contains(first) {
                            children.insert(first, Default::default());
                        }
                        return Box::pin(
                            children
                                .get_mut(first)
                                .expect("just inserted")
                                .insert(key, k_new, v_new),
                        )
                        .await;
                    } else {
                        let n = common_length(key, key_a)?;
                        let first_a = key_a[n];
                        prefix.0.0.drain(..n + 1);
                        (first_a, n)
                    }
                };
                let common = &key[..n];
                let (&first_b, key_b) = key[n..].split_first().expect("must have 1 different");
                let node_a = std::mem::take(self);
                let node_b = Self::Leaf(
                    WithPrefix::new(
                        Prefix::from(k_new.vec().strip_suffix(key_b).expect("key mismatch")),
                        k_new,
                    )?,
                    MappedExtra(WithoutHeader, v_new),
                );
                *self = Self::from_pair(common, first_a, first_b, node_a, node_b)?;
                Ok(None)
            }
            Self::Empty => {
                *self = Self::from_kv(key, k_new, v_new)?;
                Ok(None)
            }
        }
    }

    fn clear(&mut self) {
        std::mem::take(self);
    }

    fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

fn common_length(a: &[u8], b: &[u8]) -> object_rainbow::Result<usize> {
    let n = a.iter().zip(b).take_while(|(a, b)| a == b).count();
    if a.len() == n || b.len() == n {
        Err(object_rainbow::error_consistency!(
            "key is prefix of another key"
        ))
    } else {
        Ok(n)
    }
}

pub struct Amt<K, V>(Node<K, V>);

impl<K, V> Default for Amt<K, V> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K: InlineOutput + Traversible + Clone, V: InlineOutput + Traversible + Clone> Amt<K, V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get(&self, k: &K) -> object_rainbow::Result<Option<V>> {
        self.0.get(&k.vec()).await
    }

    pub async fn insert(&mut self, k: K, v: V) -> object_rainbow::Result<Option<V>> {
        self.0.insert(&k.vec(), k, v).await
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
    use object_rainbow::zero_terminated::Zt;
    use smol_macros::test;

    use crate::Amt;

    #[apply(test!)]
    async fn test() -> object_rainbow::Result<()> {
        let mut node = Amt::<[u8; 4], ()>::new();
        node.insert(*b"abcd", ()).await?;
        assert_eq!(node.get(b"abcd").await?, Some(()));
        node.insert(*b"abce", ()).await?;
        assert_eq!(node.get(b"abcd").await?, Some(()));
        assert_eq!(node.get(b"abce").await?, Some(()));
        node.insert(*b"abff", ()).await?;
        assert_eq!(node.get(b"abcd").await?, Some(()));
        assert_eq!(node.get(b"abce").await?, Some(()));
        assert_eq!(node.get(b"abff").await?, Some(()));
        node.insert(*b"abfg", ()).await?;
        assert_eq!(node.get(b"abcd").await?, Some(()));
        assert_eq!(node.get(b"abce").await?, Some(()));
        assert_eq!(node.get(b"abff").await?, Some(()));
        assert_eq!(node.get(b"abfg").await?, Some(()));
        Ok(())
    }

    #[apply(test!)]
    async fn test_apple_apricot() -> object_rainbow::Result<()> {
        let mut node = Amt::<Zt<String>, ()>::default();
        node.insert(Zt::new("apple".into())?, ()).await?;
        node.insert(Zt::new("apricot".into())?, ()).await?;
        assert_eq!(node.get(&Zt::new("apple".into())?).await?, Some(()));
        assert_eq!(node.get(&Zt::new("apricot".into())?).await?, Some(()));
        Ok(())
    }
}
