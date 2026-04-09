use object_rainbow::{
    Fetch, Inline, InlineOutput, ListHashes, Object, Parse, ParseInline, Tagged, ToOutput,
    Topological, Traversible, assert_impl,
};
use object_rainbow_append_tree::AppendTree;
use object_rainbow_point::{IntoPoint, Point};

#[derive(ToOutput, Tagged, ListHashes, Topological, Parse, Clone)]
#[topology(recursive)]
struct ChainNode<T> {
    value: T,
    #[tags(skip)]
    #[parse(unchecked)]
    #[topology(unchecked)]
    tree: AppendTree<Point<Self>>,
}

assert_impl!(
    impl<T, E> Object<E> for ChainNode<T>
    where
        E: 'static + Send + Sync + Clone,
        T: Inline<E>,
    {
    }
);

#[derive(ToOutput, InlineOutput, Tagged, ListHashes, Topological, Parse, ParseInline)]
pub struct ChainTree<T>(Option<Point<ChainNode<T>>>);

assert_impl!(
    impl<T, E> Inline<E> for ChainTree<T>
    where
        E: 'static + Send + Sync + Clone,
        T: Inline<E>,
    {
    }
);

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

impl<T: Send + Sync + Clone + Traversible + InlineOutput> ChainTree<T> {
    pub async fn push(&mut self, value: T) -> object_rainbow::Result<()> {
        let tree = if let Some(node) = self.0.take() {
            let mut tree = node.fetch().await?.tree;
            tree.push(node).await?;
            tree
        } else {
            Default::default()
        };
        self.0 = Some(ChainNode { value, tree }.point());
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
        Ok(Self(node.fetch().await?.tree.last().await?))
    }
}
