use std::{future::ready, marker::PhantomData};

use object_rainbow::{
    Enum, Fetch, Inline, InlineOutput, ListHashes, Object, Output, Parse, ParseAsInline,
    ParseInline, ParseInput, Tagged, ToOutput, Topological, Traversible, assert_impl, numeric::Le,
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

impl<T: PartialEq, N, M> PartialEq for Node<T, N, M> {
    fn eq(&self, other: &Self) -> bool {
        self.prev == other.prev && self.items == other.items
    }
}

impl<T: Eq, N, M> Eq for Node<T, N, M> {}

trait History: Sized + Send + Sync {
    type History: Send + Sync;
    type Block: Send + Sync;
    const CAPACITY: u64;
}

trait ToContiguousOutput: History {
    fn to_contiguous_output(&self, history: &Self::History, output: &mut dyn Output);
}

trait ParseWithLen<I: ParseInput>: History {
    fn parse_with_len(input: &mut I, len: u64) -> object_rainbow::Result<(Self::History, Self)>;
}

trait Push: Clone + History {
    type T: Send + Sync;
    fn get(&self, index: u64) -> impl Send + Future<Output = object_rainbow::Result<Self::T>>;
    fn push(
        &mut self,
        len: u64,
        value: Self::T,
        history: &mut Self::History,
    ) -> object_rainbow::Result<()>;
    fn last<'a>(&'a self, history: &'a Self::History) -> Option<&'a Self::T>;
    fn from_value(prev: Point<Self>, history: &mut Self::History, value: Self::T) -> Self;
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

struct Leaf;

impl<T: Send + Sync, N: Send + Sync + Unsigned> History for Node<T, N, Leaf> {
    type History = ();
    type Block = Self;
    const CAPACITY: u64 = N::U64;
}

impl<T: InlineOutput + Send + Sync, N: Send + Sync + Unsigned> ToContiguousOutput
    for Node<T, N, Leaf>
{
    fn to_contiguous_output(&self, (): &Self::History, output: &mut dyn Output) {
        self.to_output(output);
    }
}

impl<T: ParseInline<I> + Send + Sync, N: Send + Sync + Unsigned, I: ParseInput> ParseWithLen<I>
    for Node<T, N, Leaf>
where
    Point<Self>: ParseInline<I>,
{
    fn parse_with_len(input: &mut I, len: u64) -> object_rainbow::Result<(Self::History, Self)> {
        if len > N::U64 {
            return Err(object_rainbow::error_parse!("overflow"));
        }
        Ok((
            (),
            Self::new(
                input.parse_inline()?,
                input.parse_vec_n(
                    len.try_into()
                        .map_err(|_| object_rainbow::error_parse!("overflow"))?,
                )?,
            ),
        ))
    }
}

impl<T: Send + Sync + Clone + Traversible + InlineOutput, N: Send + Sync + Unsigned> Push
    for Node<T, N, Leaf>
{
    type T = T;

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
        (): &mut Self::History,
    ) -> object_rainbow::Result<()> {
        if len != (self.items.len() as u64) {
            Err(object_rainbow::error_fetch!("leaf len mismatch"))
        } else if self.items.len() >= N::USIZE {
            Err(object_rainbow::error_fetch!("leaf overflow"))
        } else {
            self.items.push(value);
            Ok(())
        }
    }

    fn last<'a>(&'a self, (): &'a Self::History) -> Option<&'a Self::T> {
        self.items.last()
    }

    fn from_value(prev: Point<Self>, (): &mut Self::History, value: Self::T) -> Self {
        Self::new(Some(prev), vec![value])
    }
}

struct NonLeaf;

impl<T: Send + Sync + History, N: Send + Sync + Unsigned> History for Node<Point<T>, N, NonLeaf> {
    type History = (T::History, T);
    type Block = T::Block;
    const CAPACITY: u64 = N::U64.saturating_mul(T::CAPACITY);
}

impl<T: ToContiguousOutput, N: Send + Sync + Unsigned> ToContiguousOutput
    for Node<Point<T>, N, NonLeaf>
{
    fn to_contiguous_output(&self, (history, prev): &Self::History, output: &mut dyn Output) {
        prev.to_contiguous_output(history, output);
        self.to_output(output);
    }
}

impl<T: Send + Sync + ParseWithLen<I>, N: Send + Sync + Unsigned, I: ParseInput> ParseWithLen<I>
    for Node<Point<T>, N, NonLeaf>
where
    Point<T>: ParseInline<I>,
    Point<Self>: ParseInline<I>,
{
    fn parse_with_len(input: &mut I, len: u64) -> object_rainbow::Result<(Self::History, Self)> {
        let own = len / T::CAPACITY
            + if len.is_multiple_of(T::CAPACITY) {
                0
            } else {
                1
            };
        if own > N::U64 {
            return Err(object_rainbow::error_parse!("overflow"));
        }
        let history = T::parse_with_len(input, len % T::CAPACITY)?;
        Ok((
            history,
            Self::new(
                input.parse_inline()?,
                input.parse_vec_n(
                    own.try_into()
                        .map_err(|_| object_rainbow::error_parse!("overflow"))?,
                )?,
            ),
        ))
    }
}

impl<T: Push + Traversible, N: Send + Sync + Unsigned> Push for Node<Point<T>, N, NonLeaf> {
    type T = T::T;

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
        (history, prev): &mut Self::History,
    ) -> object_rainbow::Result<()> {
        if let Some(last) = self.items.last_mut() {
            if len.is_multiple_of(T::CAPACITY) {
                let last = last.clone();
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
                let last = T::from_value(last, history, value);
                self.items.push(last.clone().point());
                *prev = last;
                Ok(())
            } else {
                prev.push(len % T::CAPACITY, value, history)?;
                *last = prev.clone().point();
                Ok(())
            }
        } else {
            Err(object_rainbow::error_fetch!("empty non-leaf encountered"))
        }
    }

    fn last<'a>(&'a self, (history, node): &'a Self::History) -> Option<&'a Self::T> {
        node.last(history)
    }

    fn from_value(prev: Point<Self>, (history, child): &mut Self::History, value: Self::T) -> Self {
        *child = T::from_value(child.clone().point(), history, value);
        Self::new(Some(prev), vec![child.clone().point()])
    }
}

impl<T: Push<History: Clone> + Traversible, N: Send + Sync + Unsigned> Node<Point<T>, N, NonLeaf> {
    fn from_inner(
        inner: T,
        history: &mut T::History,
        value: T::T,
    ) -> (<Self as History>::History, Self) {
        let inner = inner.point();
        let next = T::from_value(inner.clone(), history, value);
        let parent = Self::new(None, vec![inner, next.clone().point()]);
        ((history.clone(), next), parent)
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
    impl<T> Push for N1<T> where T: Send + Sync + Clone + Traversible + InlineOutput {}
);
assert_impl!(
    impl<T> Push for N2<T> where T: Send + Sync + Clone + Traversible + InlineOutput {}
);
assert_impl!(
    impl<T> Push for N3<T> where T: Send + Sync + Clone + Traversible + InlineOutput {}
);
assert_impl!(
    impl<T> Push for N4<T> where T: Send + Sync + Clone + Traversible + InlineOutput {}
);
assert_impl!(
    impl<T> Push for N5<T> where T: Send + Sync + Clone + Traversible + InlineOutput {}
);
assert_impl!(
    impl<T> Push for N6<T> where T: Send + Sync + Clone + Traversible + InlineOutput {}
);
assert_impl!(
    impl<T> Push for N7<T> where T: Send + Sync + Clone + Traversible + InlineOutput {}
);
assert_impl!(
    impl<T> Push for N8<T> where T: Send + Sync + Clone + Traversible + InlineOutput {}
);

type H1 = ();
type H2<T> = (H1, N1<T>);
type H3<T> = (H2<T>, N2<T>);
type H4<T> = (H3<T>, N3<T>);
type H5<T> = (H4<T>, N4<T>);
type H6<T> = (H5<T>, N5<T>);
type H7<T> = (H6<T>, N6<T>);
type H8<T> = (H7<T>, N7<T>);

#[derive(Enum, Tagged, ListHashes, Topological, Clone)]
enum TreeKind<T> {
    N1((H1, N1<T>)),
    N2((H2<T>, N2<T>)),
    N3((H3<T>, N3<T>)),
    N4((H4<T>, N4<T>)),
    N5((H5<T>, N5<T>)),
    N6((H6<T>, N6<T>)),
    N7((H7<T>, N7<T>)),
    N8((H8<T>, N8<T>)),
}

#[derive(Tagged, ListHashes, Topological, Clone, ParseAsInline)]
pub struct AppendTree<T> {
    len: Le<u64>,
    kind: TreeKind<T>,
}

impl<T: Send + Sync + InlineOutput> ToOutput for AppendTree<T> {
    fn to_output(&self, output: &mut dyn Output) {
        self.len.to_output(output);
        match &self.kind {
            TreeKind::N1((history, node)) => node.to_contiguous_output(history, output),
            TreeKind::N2((history, node)) => node.to_contiguous_output(history, output),
            TreeKind::N3((history, node)) => node.to_contiguous_output(history, output),
            TreeKind::N4((history, node)) => node.to_contiguous_output(history, output),
            TreeKind::N5((history, node)) => node.to_contiguous_output(history, output),
            TreeKind::N6((history, node)) => node.to_contiguous_output(history, output),
            TreeKind::N7((history, node)) => node.to_contiguous_output(history, output),
            TreeKind::N8((history, node)) => node.to_contiguous_output(history, output),
        }
    }
}

const C1: u64 = 256;
const C2: u64 = 256 * C1;
const C3: u64 = 256 * C2;
const C4: u64 = 256 * C3;
const C5: u64 = 256 * C4;
const C6: u64 = 256 * C5;
const C7: u64 = 256 * C6;
const C8: u64 = C7.saturating_mul(256);
const C2_MIN: u64 = C1 + 1;
const C3_MIN: u64 = C2 + 1;
const C4_MIN: u64 = C3 + 1;
const C5_MIN: u64 = C4 + 1;
const C6_MIN: u64 = C5 + 1;
const C7_MIN: u64 = C6 + 1;
const C8_MIN: u64 = C7 + 1;

impl<T, I: ParseInput> ParseInline<I> for AppendTree<T>
where
    N1<T>: ParseWithLen<I, History = H1>,
    N2<T>: ParseWithLen<I, History = H2<T>>,
    N3<T>: ParseWithLen<I, History = H3<T>>,
    N4<T>: ParseWithLen<I, History = H4<T>>,
    N5<T>: ParseWithLen<I, History = H5<T>>,
    N6<T>: ParseWithLen<I, History = H6<T>>,
    N7<T>: ParseWithLen<I, History = H7<T>>,
    N8<T>: ParseWithLen<I, History = H8<T>>,
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let len = input.parse_inline::<Le<u64>>()?;
        let kind = match len.0 {
            0..=C1 => TreeKind::N1(N1::<T>::parse_with_len(input, len.0)?),
            C2_MIN..=C2 => TreeKind::N2(N2::<T>::parse_with_len(input, len.0)?),
            C3_MIN..=C3 => TreeKind::N3(N3::<T>::parse_with_len(input, len.0)?),
            C4_MIN..=C4 => TreeKind::N4(N4::<T>::parse_with_len(input, len.0)?),
            C5_MIN..=C5 => TreeKind::N5(N5::<T>::parse_with_len(input, len.0)?),
            C6_MIN..=C6 => TreeKind::N6(N6::<T>::parse_with_len(input, len.0)?),
            C7_MIN..=C7 => TreeKind::N7(N7::<T>::parse_with_len(input, len.0)?),
            C8_MIN..=C8 => TreeKind::N8(N8::<T>::parse_with_len(input, len.0)?),
        };
        Ok(Self { len, kind })
    }
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
            kind: TreeKind::N1(((), Node::new(None, Vec::new()))),
        }
    }

    pub async fn get(&self, index: u64) -> object_rainbow::Result<Option<T>> {
        if index < self.len.0 {
            match &self.kind {
                TreeKind::N1((_, node)) => node.get(index).await,
                TreeKind::N2((_, node)) => node.get(index).await,
                TreeKind::N3((_, node)) => node.get(index).await,
                TreeKind::N4((_, node)) => node.get(index).await,
                TreeKind::N5((_, node)) => node.get(index).await,
                TreeKind::N6((_, node)) => node.get(index).await,
                TreeKind::N7((_, node)) => node.get(index).await,
                TreeKind::N8((_, node)) => node.get(index).await,
            }
            .map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn push(&mut self, value: T) -> object_rainbow::Result<()> {
        let len = self.len.0;
        macro_rules! upgrade {
            ($history:ident, $node:ident, $child:ident, $parent:ident) => {
                if len == $child::<T>::CAPACITY {
                    self.kind =
                        TreeKind::$parent(Node::from_inner(std::mem::take($node), $history, value));
                } else {
                    $node.push(len, value, $history)?;
                }
            };
        }
        match &mut self.kind {
            TreeKind::N1((history, node)) => upgrade!(history, node, N1, N2),
            TreeKind::N2((history, node)) => upgrade!(history, node, N2, N3),
            TreeKind::N3((history, node)) => upgrade!(history, node, N3, N4),
            TreeKind::N4((history, node)) => upgrade!(history, node, N4, N5),
            TreeKind::N5((history, node)) => upgrade!(history, node, N5, N6),
            TreeKind::N6((history, node)) => upgrade!(history, node, N6, N7),
            TreeKind::N7((history, node)) => upgrade!(history, node, N7, N8),
            TreeKind::N8((history, node)) => {
                if len == N8::<T>::CAPACITY {
                    return Err(object_rainbow::error_fetch!("root overflow"));
                } else {
                    node.push(len, value, history)?;
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

    pub fn last(&self) -> Option<&T> {
        match &self.kind {
            TreeKind::N1((history, node)) => Push::last(node, history),
            TreeKind::N2((history, node)) => Push::last(node, history),
            TreeKind::N3((history, node)) => Push::last(node, history),
            TreeKind::N4((history, node)) => Push::last(node, history),
            TreeKind::N5((history, node)) => Push::last(node, history),
            TreeKind::N6((history, node)) => Push::last(node, history),
            TreeKind::N7((history, node)) => Push::last(node, history),
            TreeKind::N8((history, node)) => Push::last(node, history),
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
            tree.push(Le(i))?;
            assert_eq!(tree.get(i).await?.unwrap().0, i);
        }
        Ok(())
    }
}
