use futures_util::{TryStreamExt, future::try_join};
use object_rainbow::{
    Enum, Equivalent, EquivalentFor, Fetch, Inline, InlineOutput, ListHashes, Parse, ParseInline,
    PointInput, Singular, Tagged, ToOutput, Topological, Traversible, assert_impl,
    length_prefixed::LpBytes, map_extra::MappedExtra, tuple_extra::Extra1,
};
use object_rainbow_array_map::KeyedArrayMap;
use object_rainbow_parse_prefix::{Prefix, PrefixRoot, WithByte, WithBytes, WithPrefix};
use object_rainbow_point::{IntoPoint, Point};

#[derive(
    Debug,
    Enum,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    Default,
    PartialEq,
    Eq,
)]
#[topology(recursive, unchecked)]
#[topology(bound = "K: InlineOutput + Traversible")]
#[topology(bound = "V: InlineOutput + Traversible")]
#[parse(input = "I", unchecked)]
#[parse(generic = "E: 'static + Send + Sync + Clone")]
#[parse(bound = "K: ParseInline<I::WithExtra<E>> + Inline<E>")]
#[parse(bound = "V: ParseInline<I::WithExtra<E>> + Inline<E>")]
#[parse(bound = "I: PointInput<Extra = (Prefix, E)>")]
enum Node<K, V> {
    #[default]
    Empty,
    Leaf(WithPrefix<K>, MappedExtra<V, Extra1>),
    Sub(#[tags(skip)] Point<Subs<K, V>>),
}

type Subs<K, V> = MappedExtra<KeyedArrayMap<MappedExtra<Node<K, V>, WithByte>>, WithBytes>;

trait _PrefixInline<E>: Inline<(Prefix, E)> {}

impl<T: Inline<(Prefix, E)>, E> _PrefixInline<E> for T {}

assert_impl!(
    impl<K, V, E> _PrefixInline<E> for Node<K, V>
    where
        E: 'static + Send + Sync + Clone,
        K: Inline<E>,
        V: Inline<E>,
    {
    }
);

impl<K, V> Node<K, V> {
    fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    fn clear(&mut self) {
        std::mem::take(self);
    }
}

impl<K: 'static, V: 'static, U: 'static + Equivalent<V>> Equivalent<Node<K, V>> for Node<K, U> {
    fn into_equivalent(self) -> Node<K, V> {
        match self {
            Self::Empty => Node::Empty,
            Self::Leaf(k, MappedExtra(e, u)) => Node::Leaf(k, MappedExtra(e, u.into_equivalent())),
            Self::Sub(point) => Node::Sub(point.into_equivalent()),
        }
    }

    fn from_equivalent(node: Node<K, V>) -> Self {
        match node {
            Node::Empty => Self::Empty,
            Node::Leaf(k, MappedExtra(e, v)) => Self::Leaf(k, MappedExtra(e, v.equivalent_for())),
            Node::Sub(point) => Node::Sub(point.equivalent_for()),
        }
    }
}

impl<K: InlineOutput + Traversible + Clone, V: InlineOutput + Traversible + Clone> Node<K, V> {
    async fn map<U: InlineOutput + Traversible + Clone>(
        self,
        f: impl Copy + Fn(V) -> U,
    ) -> object_rainbow::Result<Node<K, U>> {
        Ok(match self {
            Self::Empty => Node::Empty,
            Self::Leaf(k, MappedExtra(e, v)) => Node::Leaf(k, MappedExtra(e, f(v))),
            Self::Sub(mut point) => Node::Sub(
                {
                    let MappedExtra(prefix, KeyedArrayMap(subs)) = point.fetch_take().await?;
                    let subs = futures_util::future::try_join_all(subs.into_iter().map(
                        |(k, MappedExtra(e, node))| async move {
                            let node = node.map(f).await?;
                            Ok::<_, object_rainbow::Error>((k, MappedExtra(e, node)))
                        },
                    ))
                    .await?
                    .into_iter()
                    .collect();
                    MappedExtra(prefix, KeyedArrayMap(subs))
                }
                .point(),
            ),
        })
    }

    async fn filter_map<U: InlineOutput + Traversible + Clone>(
        self,
        f: impl Copy + Fn(V) -> Option<U>,
    ) -> object_rainbow::Result<Node<K, U>> {
        Ok(match self {
            Self::Empty => Node::Empty,
            Self::Leaf(k, MappedExtra(e, v)) => Node::Leaf(
                k,
                MappedExtra(
                    e,
                    match f(v) {
                        Some(u) => u,
                        None => return Ok(Node::Empty),
                    },
                ),
            ),
            Self::Sub(mut point) => Node::Sub(
                {
                    let MappedExtra(prefix, KeyedArrayMap(subs)) = point.fetch_take().await?;
                    let subs = futures_util::future::try_join_all(subs.into_iter().map(
                        |(k, MappedExtra(e, node))| async move {
                            let node = node.filter_map(f).await?;
                            Ok::<_, object_rainbow::Error>((k, MappedExtra(e, node)))
                        },
                    ))
                    .await?
                    .into_iter()
                    .filter(|(_, node)| !Node::is_empty(node))
                    .collect();
                    let mut subs = MappedExtra(prefix, KeyedArrayMap(subs));
                    if let Some(node) = Node::collapse(&mut subs).await? {
                        return Ok(node);
                    }
                    subs
                }
                .point(),
            ),
        })
    }

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
            MappedExtra(Default::default(), v),
        ))
    }

    fn from_pair(common: &[u8], first_a: u8, first_b: u8, node_a: Self, node_b: Self) -> Self {
        Self::Sub(
            MappedExtra(
                WithBytes(LpBytes(common.into())),
                KeyedArrayMap(
                    [
                        (first_a, MappedExtra(Default::default(), node_a)),
                        (first_b, MappedExtra(Default::default(), node_b)),
                    ]
                    .into(),
                ),
            )
            .point(),
        )
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
        if wp_a.vec() != key_a {
            return Err(object_rainbow::error_consistency!("suffix mismatch"));
        }
        let node_a = Self::Leaf(wp_a, MappedExtra(Default::default(), v_a));
        let wp_b = WithPrefix::new(prefix.with(vec![first_b]), k_b)?;
        if wp_b.vec() != key_b {
            return Err(object_rainbow::error_consistency!("suffix mismatch"));
        }
        let node_b = Self::Leaf(wp_b, MappedExtra(Default::default(), v_b));
        Ok(Self::from_pair(common, first_a, first_b, node_a, node_b))
    }

    async fn insert(
        &mut self,
        key: &[u8],
        k_new: K,
        v_new: V,
        replace: bool,
    ) -> object_rainbow::Result<Option<(K, V)>> {
        match &mut *self {
            Self::Leaf(k, MappedExtra(_, v)) => {
                let vec = k.vec();
                if vec == key {
                    if replace {
                        Ok(Some((k.replace_equal(k_new), std::mem::replace(v, v_new))))
                    } else {
                        Ok(Some((k_new, v_new)))
                    }
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
                        return Box::pin(
                            children
                                .entry(first)
                                .or_default()
                                .insert(key, k_new, v_new, replace),
                        )
                        .await;
                    }
                    let n = common_length(key, key_a)?;
                    let first_a = key_a[n];
                    prefix.0.0.drain(..n + 1);
                    (first_a, n)
                };
                let common = &key[..n];
                let (&first_b, key_b) = key[n..].split_first().expect("must have 1 different");
                let node_a = std::mem::take(self);
                let node_b = Self::Leaf(
                    WithPrefix::new(
                        Prefix::from(k_new.vec().strip_suffix(key_b).ok_or_else(|| {
                            object_rainbow::error_consistency!("suffix mismatch")
                        })?),
                        k_new,
                    )?,
                    MappedExtra(Default::default(), v_new),
                );
                *self = Self::from_pair(common, first_a, first_b, node_a, node_b);
                Ok(None)
            }
            Self::Empty => {
                *self = Self::from_kv(key, k_new, v_new)?;
                Ok(None)
            }
        }
    }

    async fn remove(&mut self, key: &[u8]) -> object_rainbow::Result<Option<(K, V)>> {
        match &mut *self {
            Self::Leaf(k, mapped_extra) if k.vec() == key => {
                let Self::Leaf(k, MappedExtra(_, v)) = std::mem::take(self) else {
                    unreachable!()
                };
                Ok(Some((k.into_value(), v)))
            }
            Self::Sub(point) => {
                let (kv, node) = {
                    let subs = &mut *point.fetch_mut().await?;
                    let kv = if let Some(key) = key.strip_prefix(&*subs.0.0.0)
                        && let Some((&first, key)) = key.split_first()
                        && let Some(sub) = subs.1.get_mut(first)
                    {
                        let kv = Box::pin(sub.remove(key)).await?;
                        if sub.is_empty() {
                            subs.1.remove(first);
                        }
                        kv
                    } else {
                        return Ok(None);
                    };
                    (kv, Self::collapse(subs).await?)
                };
                if let Some(node) = node {
                    *self = node;
                }
                Ok(kv)
            }
            _ => Ok(None),
        }
    }

    fn append(
        &mut self,
        other: &mut Self,
    ) -> impl Send + Future<Output = object_rainbow::Result<()>> {
        async move {
            match (&mut *self, &mut *other) {
                (_, Self::Empty) => {}
                (Self::Empty, _) => std::mem::swap(self, other),
                (_, Self::Leaf(_, _)) => {
                    let Self::Leaf(k, MappedExtra(_, v)) = std::mem::take(other) else {
                        unreachable!()
                    };
                    self.insert(&k.vec(), k.into_value(), v, true).await?;
                }
                (Self::Leaf(_, _), _) => {
                    let Self::Leaf(k, MappedExtra(_, v)) = std::mem::take(self) else {
                        unreachable!()
                    };
                    other.insert(&k.vec(), k.into_value(), v, false).await?;
                    std::mem::swap(self, other);
                }
                (Self::Sub(this), Self::Sub(o_point)) => {
                    if this.hash() == o_point.hash() {
                        *self = std::mem::take(other);
                        return Ok(());
                    }
                    let (mut s, mut o) = try_join(this.fetch_mut(), o_point.fetch_mut()).await?;
                    if let Some(suffix) = o.0.0.0.strip_prefix(&*s.0.0.0) {
                        if let Some((&first, _)) = suffix.split_first() {
                            o.0.0.0.drain(..s.0.0.0.len() + 1);
                            drop(o);
                            match s.1.entry(first) {
                                object_rainbow_array_map::Entry::Vacant(e) => {
                                    e.insert(MappedExtra(
                                        Default::default(),
                                        std::mem::take(other),
                                    ));
                                }
                                object_rainbow_array_map::Entry::Occupied(e) => {
                                    Box::pin(e.into_mut().append(other)).await?;
                                }
                            }
                        } else {
                            {
                                let mut futures = futures_util::stream::FuturesUnordered::new();
                                for (key, node) in s.1.iter_mut() {
                                    if let Some(mut other) = o.1.remove(key) {
                                        futures.push(async move { node.append(&mut other).await });
                                    }
                                }
                                while futures.try_next().await?.is_some() {}
                            }
                            while let Some((key, sub)) = o.1.pop_first() {
                                assert!(!s.1.contains(key));
                                s.1.insert(key, sub);
                            }
                            assert!(o.is_empty());
                            drop(o);
                            *other = Self::Empty;
                        }
                    } else if let Some(suffix) = s.0.0.0.strip_prefix(&*o.0.0.0) {
                        let Some((&first, _)) = suffix.split_first() else {
                            unreachable!()
                        };
                        s.0.0.0.drain(..o.0.0.0.len() + 1);
                        drop(s);
                        match o.1.entry(first) {
                            object_rainbow_array_map::Entry::Vacant(e) => {
                                e.insert(MappedExtra(Default::default(), std::mem::take(self)));
                            }
                            object_rainbow_array_map::Entry::Occupied(e) => {
                                let other = e.into_mut();
                                Box::pin(other.append(self)).await?;
                                std::mem::swap(self, other);
                            }
                        }
                        drop(o);
                        std::mem::swap(self, other);
                    } else {
                        let n = common_length(&s.0.0.0, &o.0.0.0)?;
                        let common = &*s.0.0.0[..n].to_vec();
                        let first_s = s.0.0.0[n];
                        let first_o = o.0.0.0[n];
                        s.0.0.0.drain(..n + 1);
                        o.0.0.0.drain(..n + 1);
                        drop(s);
                        drop(o);
                        *self = Self::from_pair(
                            common,
                            first_s,
                            first_o,
                            std::mem::take(self),
                            std::mem::take(other),
                        );
                    }
                }
            }
            assert!(other.is_empty());
            Ok(())
        }
    }

    fn append_swap(
        &mut self,
        other: &mut Self,
    ) -> impl Send + Future<Output = object_rainbow::Result<()>> {
        async move {
            match (&mut *self, &mut *other) {
                (_, Self::Empty) => {}
                (Self::Empty, _) => std::mem::swap(self, other),
                (_, Self::Leaf(_, _)) => {
                    let Self::Leaf(k, MappedExtra(_, v)) = std::mem::take(other) else {
                        unreachable!()
                    };
                    let prefix = k.prefix().clone();
                    if let Some((k, v)) = self.insert(&k.vec(), k.into_value(), v, true).await? {
                        *other = Self::Leaf(
                            WithPrefix::new(prefix, k)?,
                            MappedExtra(Default::default(), v),
                        );
                    }
                }
                (Self::Leaf(_, _), _) => {
                    let Self::Leaf(k, MappedExtra(_, v)) = std::mem::take(self) else {
                        unreachable!()
                    };
                    let prefix = k.prefix().clone();
                    if let Some((k, v)) = other.insert(&k.vec(), k.into_value(), v, false).await? {
                        *self = Self::Leaf(
                            WithPrefix::new(prefix, k)?,
                            MappedExtra(Default::default(), v),
                        );
                    }
                    std::mem::swap(self, other);
                }
                (Self::Sub(this), Self::Sub(o_point)) => {
                    if this.hash() == o_point.hash() {
                        std::mem::swap(self, other);
                        return Ok(());
                    }
                    let (mut s, mut o) = try_join(this.fetch_mut(), o_point.fetch_mut()).await?;
                    if let Some(suffix) = o.0.0.0.strip_prefix(&*s.0.0.0)
                        && let Some((&first, rest)) = suffix.split_first()
                    {
                        let rest = Vec::from(rest);
                        o.0.0.0.drain(s.0.0.0.len()..);
                        Self::make_branch(&mut o, first, &rest);
                    }
                    if let Some(suffix) = s.0.0.0.strip_prefix(&*o.0.0.0)
                        && let Some((&first, rest)) = suffix.split_first()
                    {
                        let rest = Vec::from(rest);
                        s.0.0.0.drain(o.0.0.0.len()..);
                        Self::make_branch(&mut s, first, &rest);
                    }
                    if s.0.0.0 == o.0.0.0 {
                        {
                            let mut futures = futures_util::stream::FuturesUnordered::new();
                            while let Some((key, mut n_o)) = o.1.pop_first() {
                                let mut n_s = s.1.remove(key).unwrap_or_default();
                                futures.push(async move {
                                    n_s.append_swap(&mut n_o).await?;
                                    Ok::<_, object_rainbow::Error>((key, n_s, n_o))
                                });
                            }
                            while let Some((key, n_s, n_o)) = futures.try_next().await? {
                                if !n_s.is_empty() {
                                    s.insert(key, n_s);
                                }
                                if !n_o.is_empty() {
                                    o.insert(key, n_o);
                                }
                            }
                        }
                        let n_s = Self::collapse(&mut s).await?;
                        let n_o = Self::collapse(&mut o).await?;
                        drop(s);
                        drop(o);
                        if let Some(n_s) = n_s {
                            *self = n_s;
                        }
                        if let Some(n_o) = n_o {
                            *other = n_o;
                        }
                    } else {
                        let n = common_length(&s.0.0.0, &o.0.0.0)?;
                        let common = &*s.0.0.0[..n].to_vec();
                        let first_s = s.0.0.0[n];
                        let first_o = o.0.0.0[n];
                        s.0.0.0.drain(..n + 1);
                        o.0.0.0.drain(..n + 1);
                        drop(s);
                        drop(o);
                        *self = Self::from_pair(
                            common,
                            first_s,
                            first_o,
                            std::mem::take(self),
                            std::mem::take(other),
                        );
                    }
                }
            }
            Ok(())
        }
    }

    fn op<U: InlineOutput + Traversible + Clone>(
        &mut self,
        other: &mut Node<K, U>,
        op: &impl TraitOp<K, V, U>,
    ) -> impl Send + Future<Output = object_rainbow::Result<()>> {
        async move {
            match (&mut *self, &mut *other) {
                (_, Node::Empty) => {}
                (Self::Empty, _) => *self = op.t1_empty(other).await?,
                (_, Node::Leaf(_, _)) => {
                    let Node::Leaf(k, MappedExtra(_, u)) = std::mem::take(other) else {
                        unreachable!()
                    };
                    let prefix = k.prefix().clone();
                    if let Some((k, u)) = op.single_kv2(self, &k.vec(), k.into_value(), u).await? {
                        *other = Node::Leaf(
                            WithPrefix::new(prefix, k)?,
                            MappedExtra(Default::default(), u),
                        );
                    }
                }
                (Self::Leaf(_, _), _) => {
                    let Self::Leaf(k, MappedExtra(_, v)) = std::mem::take(self) else {
                        unreachable!()
                    };
                    *self = op.single_kv1(other, &k.vec(), k.into_value(), v).await?;
                }
                (Self::Sub(this), Node::Sub(o_point)) => {
                    let (mut s, mut o) = try_join(this.fetch_mut(), o_point.fetch_mut()).await?;
                    if let Some(suffix) = o.0.0.0.strip_prefix(&*s.0.0.0)
                        && let Some((&first, rest)) = suffix.split_first()
                    {
                        let rest = Vec::from(rest);
                        o.0.0.0.drain(s.0.0.0.len()..);
                        Node::make_branch(&mut o, first, &rest);
                    }
                    if let Some(suffix) = s.0.0.0.strip_prefix(&*o.0.0.0)
                        && let Some((&first, rest)) = suffix.split_first()
                    {
                        let rest = Vec::from(rest);
                        s.0.0.0.drain(o.0.0.0.len()..);
                        Self::make_branch(&mut s, first, &rest);
                    }
                    if s.0.0.0 == o.0.0.0 {
                        {
                            let mut futures = futures_util::stream::FuturesUnordered::new();
                            while let Some((key, mut n_o)) = o.1.pop_first() {
                                let mut n_s = s.1.remove(key).unwrap_or_default();
                                futures.push(async move {
                                    n_s.op(&mut n_o, op).await?;
                                    Ok::<_, object_rainbow::Error>((key, n_s, n_o))
                                });
                            }
                            while let Some((key, n_s, n_o)) = futures.try_next().await? {
                                if !n_s.is_empty() {
                                    s.insert(key, n_s);
                                }
                                if !n_o.is_empty() {
                                    o.insert(key, n_o);
                                }
                            }
                        }
                        let n_s = Self::collapse(&mut s).await?;
                        let n_o = Node::collapse(&mut o).await?;
                        drop(s);
                        drop(o);
                        if let Some(n_s) = n_s {
                            *self = n_s;
                        }
                        if let Some(n_o) = n_o {
                            *other = n_o;
                        }
                    } else {
                        let n = common_length(&s.0.0.0, &o.0.0.0)?;
                        let common = &*s.0.0.0[..n].to_vec();
                        let first_s = s.0.0.0[n];
                        let first_o = o.0.0.0[n];
                        s.0.0.0.drain(..n + 1);
                        o.0.0.0.drain(..n + 1);
                        drop(s);
                        drop(o);
                        *self = Self::from_pair(
                            common,
                            first_s,
                            first_o,
                            std::mem::take(self),
                            op.t1_empty(other).await?,
                        );
                    }
                }
            }
            Ok(())
        }
    }

    fn make_branch(subs: &mut Subs<K, V>, first: u8, rest: &[u8]) {
        let node = Self::Sub(
            MappedExtra(
                WithBytes(LpBytes(rest.into())),
                KeyedArrayMap(std::mem::take(&mut subs.1)),
            )
            .point(),
        );
        subs.1.insert(first, MappedExtra(Default::default(), node));
    }

    async fn collapse(subs: &mut Subs<K, V>) -> object_rainbow::Result<Option<Self>> {
        Ok(if let Some(collapse_ctx) = Self::collapse_ctx(subs) {
            Some(Self::from_ctx(collapse_ctx).await?)
        } else {
            None
        })
    }

    async fn from_ctx(collapse_ctx: Option<(Vec<u8>, u8, Self)>) -> object_rainbow::Result<Self> {
        Ok(if let Some((mut prefix, first, mut child)) = collapse_ctx {
            match &mut child {
                Self::Empty => {}
                Self::Leaf(k, _) => {
                    k.pop_n(prefix.len() + 1)?;
                }
                Self::Sub(point) => {
                    let suffix = &mut point.fetch_mut().await?.0.0.0;
                    prefix.push(first);
                    prefix.append(suffix);
                    *suffix = prefix;
                }
            }
            child
        } else {
            Self::Empty
        })
    }

    fn collapse_ctx(subs: &mut Subs<K, V>) -> CollapseCtx<K, V> {
        if subs.1.len() < 2 {
            Some(
                subs.1
                    .pop_first()
                    .map(|entry| (std::mem::take(&mut subs.0.0.0), entry.0, entry.1.1)),
            )
        } else {
            None
        }
    }
}

trait TraitOp<K: Send, V, U: Send>: Send + Sync {
    fn t1_empty(
        &self,
        t2: &mut Node<K, U>,
    ) -> impl Send + Future<Output = object_rainbow::Result<Node<K, V>>>;
    fn single_kv2(
        &self,
        t1: &mut Node<K, V>,
        key: &[u8],
        k: K,
        u: U,
    ) -> impl Send + Future<Output = object_rainbow::Result<Option<(K, U)>>>;
    fn single_kv1(
        &self,
        t2: &mut Node<K, U>,
        key: &[u8],
        k: K,
        v: V,
    ) -> impl Send + Future<Output = object_rainbow::Result<Node<K, V>>>;
}

struct BulkOp;

impl<K: InlineOutput + Traversible + Clone, V: InlineOutput + Traversible + Clone>
    TraitOp<K, V, Option<V>> for BulkOp
where
    Option<V>: InlineOutput,
{
    async fn t1_empty(&self, t2: &mut Node<K, Option<V>>) -> object_rainbow::Result<Node<K, V>> {
        std::mem::take(t2).filter_map(std::convert::identity).await
    }

    async fn single_kv2(
        &self,
        t1: &mut Node<K, V>,
        key: &[u8],
        k: K,
        u: Option<V>,
    ) -> object_rainbow::Result<Option<(K, Option<V>)>> {
        let kv = if let Some(v) = u {
            t1.insert(key, k, v, true).await?
        } else {
            t1.remove(key).await?
        };
        Ok(kv.map(|(k, v)| (k, Some(v))))
    }

    async fn single_kv1(
        &self,
        t2: &mut Node<K, Option<V>>,
        key: &[u8],
        k: K,
        v: V,
    ) -> object_rainbow::Result<Node<K, V>> {
        let kv = t2.insert(key, k, Some(v), false).await?;
        let t1 = self.t1_empty(t2).await?;
        if let Some((k, v)) = kv {
            *t2 = Node::from_kv(key, k, v)?;
        }
        Ok(t1)
    }
}

type CollapseCtx<K, V> = Option<Option<(Vec<u8>, u8, Node<K, V>)>>;

fn common_length(a: &[u8], b: &[u8]) -> object_rainbow::Result<usize> {
    let n = a.iter().zip(b).take_while(|(a, b)| a == b).count();
    if a.len() == n || b.len() == n {
        Err(object_rainbow::error_consistency!(
            "path is prefix of another path",
        ))
    } else {
        Ok(n)
    }
}

#[derive(
    Debug,
    Clone,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    PartialEq,
    Eq,
)]
pub struct AmtMap<K, V>(MappedExtra<Node<K, V>, PrefixRoot>);

assert_impl!(
    impl<K, V, E> Inline<E> for AmtMap<K, V>
    where
        E: 'static + Send + Sync + Clone,
        K: Inline<E>,
        V: Inline<E>,
    {
    }
);

impl<K, V> Default for AmtMap<K, V> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K: InlineOutput + Traversible + Clone, V: InlineOutput + Traversible + Clone> AmtMap<K, V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get(&self, k: &K) -> object_rainbow::Result<Option<V>> {
        self.0.get(&k.vec()).await
    }

    pub async fn insert(&mut self, k: K, v: V) -> object_rainbow::Result<Option<V>> {
        self.0
            .insert(&k.vec(), k, v, true)
            .await
            .map(|o| o.map(|(_, v)| v))
    }

    pub async fn remove(&mut self, k: &K) -> object_rainbow::Result<Option<V>> {
        self.0.remove(&k.vec()).await.map(|r| r.map(|(_, v)| v))
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub async fn append(&mut self, other: &mut Self) -> object_rainbow::Result<()> {
        self.0.append(&mut other.0).await
    }

    pub async fn append_swap(&mut self, other: &mut Self) -> object_rainbow::Result<()> {
        self.0.append_swap(&mut other.0).await
    }

    pub async fn map<U: InlineOutput + Traversible + Clone>(
        self,
        f: impl Fn(V) -> U,
    ) -> object_rainbow::Result<AmtMap<K, U>> {
        let AmtMap(MappedExtra(e, node)) = self;
        let node = node.map(&f).await?;
        Ok(AmtMap(MappedExtra(e, node)))
    }

    pub async fn filter_map<U: InlineOutput + Traversible + Clone>(
        self,
        f: impl Fn(V) -> Option<U>,
    ) -> object_rainbow::Result<AmtMap<K, U>>
    where
        Option<U>: InlineOutput,
    {
        let AmtMap(MappedExtra(e, node)) = self;
        let node = node.filter_map(&f).await?;
        Ok(AmtMap(MappedExtra(e, node)))
    }

    pub async fn bulk(
        &mut self,
        mut bulk: AmtMap<K, Option<V>>,
    ) -> object_rainbow::Result<AmtMap<K, V>>
    where
        Option<V>: InlineOutput,
    {
        self.0.op(&mut bulk.0, &BulkOp).await?;
        bulk.filter_map(std::convert::identity).await
    }
}

impl<K: 'static, V: 'static, U: 'static + Equivalent<V>> Equivalent<AmtMap<K, V>> for AmtMap<K, U> {
    fn into_equivalent(self) -> AmtMap<K, V> {
        AmtMap(self.0.into_equivalent())
    }

    fn from_equivalent(map: AmtMap<K, V>) -> Self {
        Self(Equivalent::from_equivalent(map.0))
    }
}

#[derive(
    Debug,
    Clone,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    PartialEq,
    Eq,
)]
pub struct AmtSet<T>(AmtMap<T, ()>);

impl<T> Default for AmtSet<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: InlineOutput + Traversible + Clone> AmtSet<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn contains(&self, value: &T) -> object_rainbow::Result<bool> {
        Ok(self.0.get(value).await?.is_some())
    }

    pub async fn insert(&mut self, value: T) -> object_rainbow::Result<bool> {
        Ok(self.0.insert(value, ()).await?.is_none())
    }

    pub async fn remove(&mut self, value: &T) -> object_rainbow::Result<bool> {
        Ok(self.0.remove(value).await?.is_some())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub async fn append(&mut self, other: &mut Self) -> object_rainbow::Result<()> {
        self.0.append(&mut other.0).await
    }

    pub async fn append_swap(&mut self, other: &mut Self) -> object_rainbow::Result<()> {
        self.0.append_swap(&mut other.0).await
    }
}

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
    use object_rainbow::zero_terminated::Zt;
    use smol_macros::test;

    use crate::AmtMap;

    #[apply(test!)]
    async fn test() -> object_rainbow::Result<()> {
        let mut amt = AmtMap::<[u8; 4], ()>::new();
        amt.insert(*b"abcd", ()).await?;
        assert_eq!(amt.get(b"abcd").await?, Some(()));
        amt.insert(*b"abce", ()).await?;
        assert_eq!(amt.get(b"abcd").await?, Some(()));
        assert_eq!(amt.get(b"abce").await?, Some(()));
        amt.insert(*b"abff", ()).await?;
        assert_eq!(amt.get(b"abcd").await?, Some(()));
        assert_eq!(amt.get(b"abce").await?, Some(()));
        assert_eq!(amt.get(b"abff").await?, Some(()));
        amt.insert(*b"abfg", ()).await?;
        assert_eq!(amt.get(b"abcd").await?, Some(()));
        assert_eq!(amt.get(b"abce").await?, Some(()));
        assert_eq!(amt.get(b"abff").await?, Some(()));
        assert_eq!(amt.get(b"abfg").await?, Some(()));
        Ok(())
    }

    #[apply(test!)]
    async fn test_apple_apricot() -> object_rainbow::Result<()> {
        let mut amt = AmtMap::<Zt<String>, ()>::default();
        amt.insert(Zt::new("apple".into())?, ()).await?;
        amt.insert(Zt::new("apricot".into())?, ()).await?;
        assert_eq!(amt.get(&Zt::new("apple".into())?).await?, Some(()));
        assert_eq!(amt.get(&Zt::new("apricot".into())?).await?, Some(()));
        Ok(())
    }

    #[apply(test!)]
    async fn remove() -> object_rainbow::Result<()> {
        let mut amt = AmtMap::<[u8; 4], ()>::new();
        amt.insert(*b"abcd", ()).await?;
        amt.insert(*b"abce", ()).await?;
        amt.insert(*b"abff", ()).await?;
        amt.insert(*b"abfg", ()).await?;
        assert_eq!(amt.get(b"abcd").await?, Some(()));
        assert_eq!(amt.get(b"abce").await?, Some(()));
        assert_eq!(amt.get(b"abff").await?, Some(()));
        assert_eq!(amt.get(b"abfg").await?, Some(()));
        amt.remove(b"abce").await?;
        amt.remove(b"abff").await?;
        assert_eq!(amt.get(b"abcd").await?, Some(()));
        assert_eq!(amt.get(b"abce").await?, None);
        assert_eq!(amt.get(b"abff").await?, None);
        assert_eq!(amt.get(b"abfg").await?, Some(()));
        amt.remove(b"abcd").await?;
        amt.remove(b"abfg").await?;
        assert!(amt.is_empty());
        Ok(())
    }

    #[apply(test!)]
    async fn append_1() -> object_rainbow::Result<()> {
        let mut a = AmtMap::<[u8; 4], ()>::new();
        a.insert(*b"abcd", ()).await?;
        a.insert(*b"abff", ()).await?;
        let mut b = AmtMap::<[u8; 4], ()>::new();
        b.insert(*b"abce", ()).await?;
        b.insert(*b"abfg", ()).await?;
        a.append(&mut b).await?;
        assert!(b.is_empty());
        assert_eq!(a.get(b"abcd").await?, Some(()));
        assert_eq!(a.get(b"abce").await?, Some(()));
        assert_eq!(a.get(b"abff").await?, Some(()));
        assert_eq!(a.get(b"abfg").await?, Some(()));
        Ok(())
    }

    #[apply(test!)]
    async fn append_2() -> object_rainbow::Result<()> {
        let mut a = AmtMap::<[u8; 4], ()>::new();
        a.insert(*b"abcd", ()).await?;
        a.insert(*b"abff", ()).await?;
        a.insert(*b"abce", ()).await?;
        let mut b = AmtMap::<[u8; 4], ()>::new();
        b.insert(*b"abfg", ()).await?;
        a.append(&mut b).await?;
        assert!(b.is_empty());
        assert_eq!(a.get(b"abcd").await?, Some(()));
        assert_eq!(a.get(b"abce").await?, Some(()));
        assert_eq!(a.get(b"abff").await?, Some(()));
        assert_eq!(a.get(b"abfg").await?, Some(()));
        Ok(())
    }

    #[apply(test!)]
    async fn append_3() -> object_rainbow::Result<()> {
        let mut a = AmtMap::<[u8; 4], ()>::new();
        a.insert(*b"abcd", ()).await?;
        a.insert(*b"abce", ()).await?;
        let mut b = AmtMap::<[u8; 4], ()>::new();
        b.insert(*b"abff", ()).await?;
        b.insert(*b"abfg", ()).await?;
        a.append(&mut b).await?;
        assert!(b.is_empty());
        assert_eq!(a.get(b"abcd").await?, Some(()));
        assert_eq!(a.get(b"abce").await?, Some(()));
        assert_eq!(a.get(b"abff").await?, Some(()));
        assert_eq!(a.get(b"abfg").await?, Some(()));
        Ok(())
    }

    #[apply(test!)]
    async fn append_swap() -> object_rainbow::Result<()> {
        let mut a = AmtMap::<[u8; 4], bool>::new();
        a.insert(*b"abcd", false).await?;
        a.insert(*b"abce", false).await?;
        a.insert(*b"abff", false).await?;
        a.insert(*b"abfg", false).await?;
        a.insert(*b"xxx1", true).await?;
        a.insert(*b"xxx2", true).await?;
        let mut b = AmtMap::<[u8; 4], bool>::new();
        b.insert(*b"abcd", true).await?;
        b.insert(*b"abce", true).await?;
        b.insert(*b"abff", true).await?;
        b.insert(*b"abfg", true).await?;
        b.insert(*b"ahij", true).await?;
        b.insert(*b"xxy1", true).await?;
        b.insert(*b"xxy2", true).await?;
        a.append_swap(&mut b).await?;
        assert_eq!(a.get(b"abcd").await?, Some(true));
        assert_eq!(a.get(b"abce").await?, Some(true));
        assert_eq!(a.get(b"abff").await?, Some(true));
        assert_eq!(a.get(b"abfg").await?, Some(true));
        assert_eq!(a.get(b"ahij").await?, Some(true));
        assert_eq!(a.get(b"xxx1").await?, Some(true));
        assert_eq!(a.get(b"xxx2").await?, Some(true));
        assert_eq!(a.get(b"xxy1").await?, Some(true));
        assert_eq!(a.get(b"xxy2").await?, Some(true));

        assert_eq!(b.get(b"abcd").await?, Some(false));
        assert_eq!(b.get(b"abce").await?, Some(false));
        assert_eq!(b.get(b"abff").await?, Some(false));
        assert_eq!(b.get(b"abfg").await?, Some(false));
        assert_eq!(b.get(b"xxx1").await?, None);
        assert_eq!(b.get(b"xxx2").await?, None);
        assert_eq!(b.get(b"xxy1").await?, None);
        assert_eq!(b.get(b"xxy2").await?, None);
        Ok(())
    }

    #[apply(test!)]
    async fn bulk() -> object_rainbow::Result<()> {
        let mut a = AmtMap::<[u8; 4], bool>::new();
        a.insert(*b"abcd", false).await?;
        a.insert(*b"abce", false).await?;
        a.insert(*b"abff", false).await?;
        a.insert(*b"abfg", false).await?;
        a.insert(*b"xxx1", true).await?;
        a.insert(*b"xxx2", true).await?;
        a.insert(*b"xxx3", false).await?;
        let mut b = AmtMap::<[u8; 4], Option<bool>>::new();
        b.insert(*b"abcd", Some(true)).await?;
        b.insert(*b"abce", Some(true)).await?;
        b.insert(*b"abff", Some(true)).await?;
        b.insert(*b"abfg", Some(true)).await?;
        b.insert(*b"ahij", Some(true)).await?;
        b.insert(*b"xxy1", Some(true)).await?;
        b.insert(*b"xxy2", Some(true)).await?;
        b.insert(*b"xxx3", None).await?;
        let b = a.bulk(b).await?;
        assert_eq!(a.get(b"abcd").await?, Some(true));
        assert_eq!(a.get(b"abce").await?, Some(true));
        assert_eq!(a.get(b"abff").await?, Some(true));
        assert_eq!(a.get(b"abfg").await?, Some(true));
        assert_eq!(a.get(b"ahij").await?, Some(true));
        assert_eq!(a.get(b"xxx1").await?, Some(true));
        assert_eq!(a.get(b"xxx2").await?, Some(true));
        assert_eq!(a.get(b"xxy1").await?, Some(true));
        assert_eq!(a.get(b"xxy2").await?, Some(true));
        assert_eq!(a.get(b"xxx3").await?, None);

        assert_eq!(b.get(b"abcd").await?, Some(false));
        assert_eq!(b.get(b"abce").await?, Some(false));
        assert_eq!(b.get(b"abff").await?, Some(false));
        assert_eq!(b.get(b"abfg").await?, Some(false));
        assert_eq!(b.get(b"xxx1").await?, None);
        assert_eq!(b.get(b"xxx2").await?, None);
        assert_eq!(b.get(b"xxy1").await?, None);
        assert_eq!(b.get(b"xxy2").await?, None);
        assert_eq!(b.get(b"xxx3").await?, Some(false));
        Ok(())
    }
}
