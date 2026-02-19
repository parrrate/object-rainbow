use futures_util::Stream;
use genawaiter_try_stream::try_stream;
use object_rainbow::{
    Fetch, Inline, InlineOutput, ListHashes, MaybeHasNiche, Object, Parse, ParseInline, Size,
    Tagged, ToOutput, Topological, Traversible, assert_impl,
};
use object_rainbow_append_tree::AppendTree;
use object_rainbow_point::{IntoPoint, Point};

#[derive(ToOutput, Tagged, ListHashes, Topological, Parse, Clone)]
#[topology(recursive)]
pub struct ChainNode<T> {
    #[tags(skip)]
    #[parse(unchecked)]
    #[topology(unchecked)]
    tree: AppendTree<Point<Self>>,
    value: T,
}

assert_impl!(
    impl<T, E> Object<E> for ChainNode<T>
    where
        E: 'static + Send + Sync + Clone,
        T: Object<E>,
    {
    }
);

impl<T: Clone + Traversible> ChainNode<T> {
    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn prev(&self) -> ChainTree<T> {
        ChainTree(self.tree.last().cloned())
    }

    pub fn into_tree(self) -> ChainTree<T> {
        ChainTree(Some(self.point()))
    }
}

#[derive(
    ToOutput, InlineOutput, Tagged, ListHashes, Topological, Parse, ParseInline, Size, MaybeHasNiche,
)]
pub struct ChainTree<T>(Option<Point<ChainNode<T>>>);

assert_impl!(
    impl<T, E> Inline<E> for ChainTree<T>
    where
        E: 'static + Send + Sync + Clone,
        T: Object<E>,
    {
    }
);

impl<T> PartialEq for ChainTree<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for ChainTree<T> {}

impl<T> Clone for ChainTree<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> ChainTree<T> {
    const EMPTY: Self = Self(None);

    pub const fn new() -> Self {
        Self::EMPTY
    }
}

impl<T> Default for ChainTree<T> {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl<T: Clone + Traversible> ChainTree<T> {
    async fn next_tree(self) -> object_rainbow::Result<AppendTree<Point<ChainNode<T>>>> {
        Ok(if let Some(node) = self.0 {
            let mut tree = node.fetch().await?.tree;
            tree.push(node)?;
            tree
        } else {
            Default::default()
        })
    }

    pub async fn with_value(self, value: T) -> object_rainbow::Result<ChainNode<T>> {
        let tree = self.next_tree().await?;
        Ok(ChainNode { value, tree })
    }

    pub async fn push(&mut self, value: T) -> object_rainbow::Result<()> {
        *self = std::mem::take(self).with_value(value).await?.into_tree();
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    pub async fn len(&self) -> object_rainbow::Result<u64> {
        let Some(node) = &self.0 else {
            return Ok(0);
        };
        node.fetch()
            .await?
            .tree
            .len()
            .checked_add(1)
            .ok_or_else(|| object_rainbow::error_fetch!("len overflow"))
    }

    pub async fn last(&self) -> object_rainbow::Result<Option<T>> {
        let Some(node) = &self.0 else {
            return Ok(None);
        };
        Ok(Some(node.fetch().await?.value))
    }

    pub async fn prev(&self) -> object_rainbow::Result<Self> {
        let Some(node) = &self.0 else {
            return Ok(Self::EMPTY);
        };
        Ok(node.fetch().await?.prev())
    }

    pub async fn follows(&self, other: &Self) -> object_rainbow::Result<bool> {
        let Some(late) = &self.0 else {
            return Ok(false);
        };
        let Some(early) = &other.0 else {
            return Ok(true);
        };
        let (late, node) = futures_util::try_join!(late.fetch(), early.fetch())?;
        let follows = late
            .tree
            .get(node.tree.len())
            .await?
            .is_some_and(|point| point == *early);
        Ok(follows)
    }

    pub async fn precedes(&self, other: &Self) -> object_rainbow::Result<bool> {
        other.follows(self).await
    }

    /// Yield entries `since`. Order is unspecified.
    pub fn diff(&self, since: &Self) -> impl Stream<Item = object_rainbow::Result<ChainNode<T>>> {
        try_stream(async move |co| {
            if self == since {
                return Ok(());
            }
            if !self.follows(since).await? {
                return Err(object_rainbow::error_fetch!("divergent histories"));
            }
            let mut current = self.clone();
            while current != *since
                && let Some(node) = current.0
            {
                let node = node.fetch().await?;
                current.0 = node.tree.last().cloned();
                co.yield_(node).await;
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
    use object_rainbow::length_prefixed::LpString;
    use smol_macros::test;

    use crate::ChainTree;

    #[apply(test!)]
    async fn test() -> object_rainbow::Result<()> {
        let root = ChainTree::<LpString>::new();
        assert!(!root.follows(&root).await?);
        let mut a = root.clone();
        a.push(LpString("a".into())).await?;
        assert!(a.follows(&root).await?);
        assert!(!a.follows(&a).await?);
        assert!(!root.follows(&a).await?);
        let mut b = root.clone();
        b.push(LpString("b".into())).await?;
        assert!(!a.follows(&b).await?);
        assert!(!b.follows(&a).await?);
        let mut ac = a.clone();
        ac.push(LpString("c".into())).await?;
        assert!(ac.follows(&root).await?);
        assert!(ac.follows(&a).await?);
        assert!(!ac.follows(&ac).await?);
        assert!(!ac.follows(&b).await?);
        assert!(!root.follows(&ac).await?);
        assert!(!a.follows(&ac).await?);
        assert!(!b.follows(&ac).await?);
        Ok(())
    }
}
