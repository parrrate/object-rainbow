use std::{future::ready, marker::PhantomData};

use object_rainbow::{
    Enum, Fetch, Inline, InlineOutput, ListHashes, Object, Parse, ParseAsInline, ParseInline,
    ParseInput, Tagged, ToOutput, Topological, Traversible, assert_impl, numeric::Le,
};
use object_rainbow_point::{IntoPoint, Point};
use typenum::{U256, Unsigned};

#[derive(ToOutput, InlineOutput, Tagged, ListHashes, Topological, Parse)]
#[topology(recursive)]
struct Node<T, N, M> {
    _capacity: PhantomData<N>,
    _marker: PhantomData<M>,
    #[tags(skip)]
    #[parse(unchecked)]
    #[topology(unchecked)]
    prev: Option<Point<Self>>,
    items: Vec<T>,
}

assert_impl!(
    impl<E, T, N, M> Object<E> for Node<T, N, M>
    where
        E: 'static + Send + Sync + Clone,
        N: 'static + Send + Sync + Clone,
        M: 'static + Send + Sync + Clone,
        T: Inline<E>,
    {
    }
);

#[derive(ToOutput, Tagged, ListHashes, Topological, ParseAsInline)]
struct HistoryNode<T, N, M> {
    count: u8,
    node: Node<T, N, M>,
}

impl<T: Clone, N, M> Clone for HistoryNode<T, N, M> {
    fn clone(&self) -> Self {
        Self {
            count: self.count,
            node: self.node.clone(),
        }
    }
}

impl<T: InlineOutput, N, M> InlineOutput for HistoryNode<T, N, M> {}

impl<T: ParseInline<I>, N, M, I: ParseInput> ParseInline<I> for HistoryNode<T, N, M>
where
    Point<Node<T, N, M>>: ParseInline<I>,
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let count = input.parse_inline::<u8>()?;
        let prev = input.parse_inline()?;
        let items = (0..=count)
            .map(|_| input.parse_inline::<T>())
            .collect::<object_rainbow::Result<_>>()?;
        Ok(Self {
            count,
            node: Node::new(prev, items),
        })
    }
}

assert_impl!(
    impl<E, T, N, M> Inline<E> for HistoryNode<T, N, M>
    where
        E: 'static + Send + Sync + Clone,
        N: 'static + Send + Sync + Clone,
        M: 'static + Send + Sync + Clone,
        T: Inline<E>,
    {
    }
);

trait JustNode: Sized + Send + Sync + Clone {
    fn new(prev: Option<Point<Self>>) -> Self;
}

trait ListNode: JustNode {
    type T: Send + Sync;
    type History: Send + Sync;
    type NextHistory: Send + Sync;
    const CAPACITY: u64;
    fn get(&self, index: u64) -> impl Send + Future<Output = object_rainbow::Result<Self::T>>;
    fn push(
        &mut self,
        len: u64,
        value: Self::T,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::History>>;
    fn merge_history(self, history: Self::History) -> Self::NextHistory;
    fn split_history(history: &Self::NextHistory) -> (&Self, &Self::History);
    fn last(&self, history: &Self::History) -> Option<Self::T>;
    fn from_value(
        prev: Point<Self>,
        history: &Self::History,
        value: Self::T,
    ) -> (Self, Self::History);
}

impl<T: Clone, N, M> Clone for Node<T, N, M> {
    fn clone(&self) -> Self {
        Self {
            _capacity: PhantomData,
            _marker: PhantomData,
            prev: self.prev.clone(),
            items: self.items.clone(),
        }
    }
}

impl<T, N, M> Default for Node<T, N, M> {
    fn default() -> Self {
        Self::new(None, Vec::new())
    }
}

impl<T, N, M> Node<T, N, M> {
    const fn new(prev: Option<Point<Self>>, items: Vec<T>) -> Self {
        Self {
            _capacity: PhantomData,
            _marker: PhantomData,
            prev,
            items,
        }
    }
}

impl<T: Send + Sync + Clone, N: Send + Sync + Unsigned, M: Send + Sync> JustNode for Node<T, N, M> {
    fn new(prev: Option<Point<Self>>) -> Self {
        Self::new(prev, Vec::new())
    }
}

struct Leaf;

impl<T: Send + Sync + Clone + Traversible + InlineOutput, N: Send + Sync + Unsigned> ListNode
    for Node<T, N, Leaf>
{
    type T = T;
    type History = ();
    type NextHistory = (HistoryNode<T, N, Leaf>, Self::History);
    const CAPACITY: u64 = N::U64;

    fn get(&self, index: u64) -> impl Send + Future<Output = object_rainbow::Result<Self::T>> {
        ready(
            usize::try_from(index)
                .map_err(|_| object_rainbow::Error::UnsupportedLength)
                .and_then(|index| {
                    self.items
                        .get(index)
                        .cloned()
                        .ok_or_else(|| object_rainbow::error_fetch!("out of bounds"))
                }),
        )
    }

    fn push(
        &mut self,
        len: u64,
        value: Self::T,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::History>> {
        ready(if len != (self.items.len() as u64) {
            Err(object_rainbow::error_fetch!("leaf len mismatch"))
        } else if self.items.len() >= N::USIZE {
            Err(object_rainbow::error_fetch!("leaf overflow"))
        } else {
            self.items.push(value);
            Ok(())
        })
    }

    fn merge_history(self, history: Self::History) -> Self::NextHistory {
        assert_ne!(self.items.len(), 0);
        assert!(self.items.len() <= 0x100);
        (
            HistoryNode {
                count: (self.items.len() - 1) as u8,
                node: self,
            },
            history,
        )
    }

    fn split_history(
        (HistoryNode { node, .. }, history): &Self::NextHistory,
    ) -> (&Self, &Self::History) {
        (node, history)
    }

    fn last(&self, (): &Self::History) -> Option<Self::T> {
        self.items.last().cloned()
    }

    fn from_value(prev: Point<Self>, (): &Self::History, value: Self::T) -> (Self, Self::History) {
        (Self::new(Some(prev), vec![value]), ())
    }
}

struct NonLeaf;

impl<T: ListNode + Traversible, N: Send + Sync + Unsigned> ListNode for Node<Point<T>, N, NonLeaf> {
    type T = T::T;
    type History = T::NextHistory;
    type NextHistory = (HistoryNode<Point<T>, N, NonLeaf>, Self::History);
    const CAPACITY: u64 = N::U64.saturating_mul(T::CAPACITY);

    fn get(&self, index: u64) -> impl Send + Future<Output = object_rainbow::Result<Self::T>> {
        async move {
            self.items
                .get(
                    usize::try_from(index / T::CAPACITY)
                        .map_err(|_| object_rainbow::Error::UnsupportedLength)?,
                )
                .ok_or_else(|| object_rainbow::error_fetch!("out of bounds"))?
                .fetch()
                .await?
                .get(index % T::CAPACITY)
                .await
        }
    }

    fn push(
        &mut self,
        len: u64,
        value: Self::T,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::History>> {
        async move {
            if let Some(last) = self.items.last_mut() {
                if len.is_multiple_of(T::CAPACITY) {
                    let prev = Some(last.clone());
                    if len / T::CAPACITY != (self.items.len() as u64) {
                        return Err(object_rainbow::error_fetch!("non-leaf len mismatch"));
                    }
                    if self.items.len() >= N::USIZE {
                        return Err(object_rainbow::error_fetch!(
                            "non-leaf overflow: {}/{}",
                            self.items.len(),
                            Self::CAPACITY,
                        ));
                    }
                    let mut last = T::new(prev);
                    let history = last.push(0, value).await?;
                    self.items.push(last.clone().point());
                    Ok(last.merge_history(history))
                } else {
                    let history = last
                        .fetch_mut()
                        .await?
                        .push(len % T::CAPACITY, value)
                        .await?;
                    Ok(last.fetch().await?.merge_history(history))
                }
            } else {
                let prev = if let Some(prev) = self.prev.as_ref() {
                    prev.fetch().await?.items.last().cloned()
                } else {
                    None
                };
                let mut first = T::new(prev);
                let history = first.push(len, value).await?;
                self.items.push(first.clone().point());
                Ok(first.merge_history(history))
            }
        }
    }

    fn merge_history(self, history: Self::History) -> Self::NextHistory {
        assert_ne!(self.items.len(), 0);
        assert!(self.items.len() <= 0x100);
        (
            HistoryNode {
                count: (self.items.len() - 1) as u8,
                node: self,
            },
            history,
        )
    }

    fn split_history(
        (HistoryNode { node, .. }, history): &Self::NextHistory,
    ) -> (&Self, &Self::History) {
        (node, history)
    }

    fn last(&self, history: &Self::History) -> Option<Self::T> {
        let (node, history) = T::split_history(history);
        node.last(history)
    }

    fn from_value(
        prev: Point<Self>,
        history: &Self::History,
        value: Self::T,
    ) -> (Self, Self::History) {
        let (child, history) = T::split_history(history);
        let (child, history) = T::from_value(child.clone().point(), history, value);
        let parent = Self::new(Some(prev), vec![child.clone().point()]);
        (parent, child.merge_history(history))
    }
}

impl<T: ListNode + Traversible, N: Send + Sync + Unsigned> Node<Point<T>, N, NonLeaf> {
    fn from_inner(
        inner: T,
        history: &T::History,
        value: T::T,
    ) -> (Self, <Self as ListNode>::History) {
        let inner = inner.point();
        let (next, history) = T::from_value(inner.clone(), history, value);
        let parent = Self::new(None, vec![inner, next.clone().point()]);
        (parent, next.merge_history(history))
    }
}

type N1<T> = Node<T, U256, Leaf>;
type N2<T> = Node<Point<N1<T>>, U256, NonLeaf>;
type N3<T> = Node<Point<N2<T>>, U256, NonLeaf>;
type N4<T> = Node<Point<N3<T>>, U256, NonLeaf>;
type N5<T> = Node<Point<N4<T>>, U256, NonLeaf>;
type N6<T> = Node<Point<N5<T>>, U256, NonLeaf>;
type N7<T> = Node<Point<N6<T>>, U256, NonLeaf>;
type N8<T> = Node<Point<N7<T>>, U256, NonLeaf>;

assert_impl!(
    impl<T> ListNode for N1<T> where T: Send + Sync + Clone + Traversible + InlineOutput {}
);
assert_impl!(
    impl<T> ListNode for N2<T> where T: Send + Sync + Clone + Traversible + InlineOutput {}
);
assert_impl!(
    impl<T> ListNode for N3<T> where T: Send + Sync + Clone + Traversible + InlineOutput {}
);
assert_impl!(
    impl<T> ListNode for N4<T> where T: Send + Sync + Clone + Traversible + InlineOutput {}
);
assert_impl!(
    impl<T> ListNode for N5<T> where T: Send + Sync + Clone + Traversible + InlineOutput {}
);
assert_impl!(
    impl<T> ListNode for N6<T> where T: Send + Sync + Clone + Traversible + InlineOutput {}
);
assert_impl!(
    impl<T> ListNode for N7<T> where T: Send + Sync + Clone + Traversible + InlineOutput {}
);
assert_impl!(
    impl<T> ListNode for N8<T> where T: Send + Sync + Clone + Traversible + InlineOutput {}
);

type H1 = ();
type H2<T> = (HistoryNode<T, U256, Leaf>, H1);
type H3<T> = (HistoryNode<Point<N1<T>>, U256, NonLeaf>, H2<T>);
type H4<T> = (HistoryNode<Point<N2<T>>, U256, NonLeaf>, H3<T>);
type H5<T> = (HistoryNode<Point<N3<T>>, U256, NonLeaf>, H4<T>);
type H6<T> = (HistoryNode<Point<N4<T>>, U256, NonLeaf>, H5<T>);
type H7<T> = (HistoryNode<Point<N5<T>>, U256, NonLeaf>, H6<T>);
type H8<T> = (HistoryNode<Point<N6<T>>, U256, NonLeaf>, H7<T>);

assert_impl!(
    impl<T, E> Inline<E> for H1
    where
        E: 'static + Send + Sync + Clone,
        T: Inline<E>,
    {
    }
);
assert_impl!(
    impl<T, E> Inline<E> for H2<T>
    where
        E: 'static + Send + Sync + Clone,
        T: Inline<E>,
    {
    }
);
assert_impl!(
    impl<T, E> Inline<E> for H3<T>
    where
        E: 'static + Send + Sync + Clone,
        T: Inline<E>,
    {
    }
);
assert_impl!(
    impl<T, E> Inline<E> for H4<T>
    where
        E: 'static + Send + Sync + Clone,
        T: Inline<E>,
    {
    }
);
assert_impl!(
    impl<T, E> Inline<E> for H5<T>
    where
        E: 'static + Send + Sync + Clone,
        T: Inline<E>,
    {
    }
);
assert_impl!(
    impl<T, E> Inline<E> for H6<T>
    where
        E: 'static + Send + Sync + Clone,
        T: Inline<E>,
    {
    }
);
assert_impl!(
    impl<T, E> Inline<E> for H7<T>
    where
        E: 'static + Send + Sync + Clone,
        T: Inline<E>,
    {
    }
);
assert_impl!(
    impl<T, E> Inline<E> for H8<T>
    where
        E: 'static + Send + Sync + Clone,
        T: Inline<E>,
    {
    }
);

#[derive(Enum, ToOutput, Tagged, ListHashes, Topological, Parse, Clone)]
enum TreeKind<T> {
    N1(H1, N1<T>),
    N2(H2<T>, N2<T>),
    N3(H3<T>, N3<T>),
    N4(H4<T>, N4<T>),
    N5(H5<T>, N5<T>),
    N6(H6<T>, N6<T>),
    N7(H7<T>, N7<T>),
    N8(H8<T>, N8<T>),
}

assert_impl!(
    impl<T, E> Object<E> for TreeKind<T>
    where
        E: 'static + Send + Sync + Clone,
        T: Inline<E>,
    {
    }
);

#[derive(ToOutput, Tagged, ListHashes, Topological, Parse, Clone)]
pub struct AppendTree<T> {
    len: Le<u64>,
    kind: TreeKind<T>,
}

assert_impl!(
    impl<T, E> Object<E> for AppendTree<T>
    where
        E: 'static + Send + Sync + Clone,
        T: Inline<E>,
    {
    }
);

impl<T: Send + Sync + Clone + Traversible + InlineOutput> AppendTree<T> {
    pub const fn new() -> Self {
        Self {
            len: Le::<u64>::new(0u64),
            kind: TreeKind::N1((), Node::new(None, Vec::new())),
        }
    }

    pub async fn get(&self, index: u64) -> object_rainbow::Result<Option<T>> {
        if index < self.len.0 {
            match &self.kind {
                TreeKind::N1(_, node) => node.get(index).await,
                TreeKind::N2(_, node) => node.get(index).await,
                TreeKind::N3(_, node) => node.get(index).await,
                TreeKind::N4(_, node) => node.get(index).await,
                TreeKind::N5(_, node) => node.get(index).await,
                TreeKind::N6(_, node) => node.get(index).await,
                TreeKind::N7(_, node) => node.get(index).await,
                TreeKind::N8(_, node) => node.get(index).await,
            }
            .map(Some)
        } else {
            Ok(None)
        }
    }

    pub async fn push(&mut self, value: T) -> object_rainbow::Result<()> {
        let len = self.len.0;
        macro_rules! upgrade {
            ($history:ident, $node:ident, $child:ident, $parent:ident) => {
                if len == $child::<T>::CAPACITY {
                    let (parent, history) =
                        Node::from_inner(std::mem::take($node), $history, value);
                    self.kind = TreeKind::$parent(history, parent);
                } else {
                    *$history = $node.push(len, value).await?;
                }
            };
        }
        match &mut self.kind {
            TreeKind::N1(history, node) => upgrade!(history, node, N1, N2),
            TreeKind::N2(history, node) => upgrade!(history, node, N2, N3),
            TreeKind::N3(history, node) => upgrade!(history, node, N3, N4),
            TreeKind::N4(history, node) => upgrade!(history, node, N4, N5),
            TreeKind::N5(history, node) => upgrade!(history, node, N5, N6),
            TreeKind::N6(history, node) => upgrade!(history, node, N6, N7),
            TreeKind::N7(history, node) => upgrade!(history, node, N7, N8),
            TreeKind::N8(history, node) => {
                if len == N8::<T>::CAPACITY {
                    return Err(object_rainbow::error_fetch!("root overflow"));
                } else {
                    *history = node.push(len, value).await?;
                }
            }
        }
        self.len.0 += 1;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.len.0 == 0
    }

    pub fn len(&self) -> u64 {
        self.len.0
    }

    pub fn last(&self) -> Option<T> {
        match &self.kind {
            TreeKind::N1(history, node) => ListNode::last(node, history),
            TreeKind::N2(history, node) => ListNode::last(node, history),
            TreeKind::N3(history, node) => ListNode::last(node, history),
            TreeKind::N4(history, node) => ListNode::last(node, history),
            TreeKind::N5(history, node) => ListNode::last(node, history),
            TreeKind::N6(history, node) => ListNode::last(node, history),
            TreeKind::N7(history, node) => ListNode::last(node, history),
            TreeKind::N8(history, node) => ListNode::last(node, history),
        }
    }
}

impl<T: Send + Sync + Clone + Traversible + InlineOutput> Default for AppendTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
    use object_rainbow::numeric::Le;
    use smol_macros::test;

    use crate::AppendTree;

    #[apply(test!)]
    async fn test() -> object_rainbow::Result<()> {
        let mut tree = AppendTree::<Le<u64>>::new();
        for i in 0..100000u64 {
            tree.push(Le(i)).await?;
            assert_eq!(tree.get(i).await?.unwrap().0, i);
        }
        Ok(())
    }
}
