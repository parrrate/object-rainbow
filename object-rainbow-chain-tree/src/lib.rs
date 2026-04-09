use std::fmt::Debug;

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

pub struct ChainHandle<T>(Option<ChainNode<T>>);

impl<T: Clone + Traversible> ChainHandle<T> {
    fn next_tree(&mut self) -> object_rainbow::Result<AppendTree<Point<ChainNode<T>>>> {
        Ok(if let Some(node) = self.0.take() {
            let mut tree = node.tree.clone();
            tree.push(node.point())?;
            tree
        } else {
            Default::default()
        })
    }

    fn with_value(&mut self, value: T) -> object_rainbow::Result<ChainNode<T>> {
        let tree = self.next_tree()?;
        Ok(ChainNode { value, tree })
    }

    pub fn push(&mut self, value: T) -> object_rainbow::Result<()> {
        self.0 = Some(self.with_value(value)?);
        Ok(())
    }

    pub fn into_tree(self) -> ChainTree<T> {
        ChainTree(self.0.map(|node| node.point()))
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

impl<T> Debug for ChainTree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ChainTree").field(&self.0).finish()
    }
}

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
    pub const EMPTY: Self = Self(None);

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
    pub fn from_values(values: impl IntoIterator<Item = T>) -> object_rainbow::Result<Self> {
        let mut values = values.into_iter();
        let Some(value) = values.next() else {
            return Ok(Self::EMPTY);
        };
        let mut node = ChainNode {
            tree: AppendTree::new(),
            value,
        };
        for value in values {
            let mut tree = node.tree.clone();
            tree.push(node.point())?;
            node = ChainNode { tree, value };
        }
        Ok(Self(Some(node.point())))
    }

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

    pub async fn slice(&self, n: u64) -> object_rainbow::Result<Self> {
        let Some(n) = n.checked_sub(1) else {
            return Ok(Self::EMPTY);
        };
        let len = self.len().await?;
        if n + 1 >= len {
            return Ok(self.clone());
        }
        let Some(node) = self.0.as_ref() else {
            return Ok(Self::EMPTY);
        };
        let tree = node.fetch().await?.tree.get(n).await?;
        Ok(Self(tree))
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
        if self == other {
            return Ok(true);
        }
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
        self.diff_backwards(since)
    }

    pub fn diff_backwards(
        &self,
        since: &Self,
    ) -> impl Stream<Item = object_rainbow::Result<ChainNode<T>>> {
        try_stream(async move |co| {
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

    pub async fn common_ancestor(&self, other: &[&Self]) -> object_rainbow::Result<Self> {
        if other.iter().all(|other| *other == self) {
            return Ok(self.clone());
        }
        let mut lo = 0;
        let mut hi = self.len().await?;
        for other in other {
            hi = hi.min(other.len().await?);
        }
        hi = hi.saturating_add(1);
        while let diff = (hi - lo) / 2
            && diff > 0
        {
            let mid = lo + diff;
            let tree = self.slice(mid).await?;
            let mut precedes = true;
            for other in other {
                precedes = precedes && tree.precedes(other).await?;
            }
            if precedes {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        self.slice(lo).await
    }
}

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
    use smol_macros::test;

    use crate::ChainTree;

    #[apply(test!)]
    async fn follows() -> object_rainbow::Result<()> {
        let root = ChainTree::<char>::new();
        assert!(root.follows(&root).await?);
        let mut a = root.clone();
        a.push('a').await?;
        assert!(a.follows(&root).await?);
        assert!(a.follows(&a).await?);
        assert!(!root.follows(&a).await?);
        let mut b = root.clone();
        b.push('b').await?;
        assert!(!a.follows(&b).await?);
        assert!(!b.follows(&a).await?);
        let mut ac = a.clone();
        ac.push('c').await?;
        assert!(ac.follows(&root).await?);
        assert!(ac.follows(&a).await?);
        assert!(ac.follows(&ac).await?);
        assert!(!ac.follows(&b).await?);
        assert!(!root.follows(&ac).await?);
        assert!(!a.follows(&ac).await?);
        assert!(!b.follows(&ac).await?);
        Ok(())
    }

    #[apply(test!)]
    async fn slice() -> object_rainbow::Result<()> {
        let root = ChainTree::<char>::from_values([])?;
        assert_eq!(root.slice(0).await?, root);
        let a = ChainTree::<char>::from_values(['a'])?;
        assert_eq!(a.slice(1).await?, a);
        assert_eq!(a.slice(0).await?, root);
        let ab = ChainTree::<char>::from_values(['a', 'b'])?;
        assert_eq!(ab.slice(2).await?, ab);
        assert_eq!(ab.slice(1).await?, a);
        assert_eq!(ab.slice(0).await?, root);
        Ok(())
    }

    #[apply(test!)]
    async fn common_ancestor() -> object_rainbow::Result<()> {
        let root = ChainTree::<char>::from_values([])?;
        let a = ChainTree::<char>::from_values(['a'])?;
        let ab = ChainTree::<char>::from_values(['a', 'b'])?;
        let ac = ChainTree::<char>::from_values(['a', 'c'])?;
        let b = ChainTree::<char>::from_values(['b'])?;
        assert_eq!(ab.common_ancestor(&[]).await?, ab);
        assert_eq!(ab.common_ancestor(&[&root]).await?, root);
        assert_eq!(ab.common_ancestor(&[&ac]).await?, a);
        assert_eq!(ac.common_ancestor(&[&ab]).await?, a);
        assert_eq!(b.common_ancestor(&[&ab]).await?, root);
        assert_eq!(ab.common_ancestor(&[&a]).await?, a);
        assert_eq!(a.common_ancestor(&[&ab]).await?, a);
        Ok(())
    }
}
