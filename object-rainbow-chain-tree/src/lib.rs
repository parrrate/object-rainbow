use object_rainbow::{
    Inline, InlineOutput, ListHashes, Object, Parse, ParseInline, Tagged, ToOutput, Topological,
    assert_impl,
};
use object_rainbow_append_tree::AppendTree;
use object_rainbow_point::Point;

#[derive(ToOutput, Tagged, ListHashes, Topological, Parse)]
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
