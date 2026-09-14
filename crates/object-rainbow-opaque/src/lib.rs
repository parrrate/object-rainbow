use std::sync::Arc;

use object_rainbow::{
    AsAny, ExtraFor, Fetch, FetchBytes, Hash, ListHashes, Output, Parse, PointInput, PointVisitor,
    Singular, SingularFetch, Tagged, ToOutput, Topological, Traversible,
    object_marker::ObjectMarker,
};

pub trait ToOutputDyn {
    fn to_output_dyn(&self, output: &mut dyn Output);
}

impl<T: ?Sized + ToOutput> ToOutputDyn for T {
    fn to_output_dyn(&self, output: &mut dyn Output) {
        self.to_output(output);
    }
}

pub trait ListHashesDyn {
    fn list_hashes_dyn(&self, f: &mut dyn FnMut(Hash));
}

impl<T: ?Sized + ListHashes> ListHashesDyn for T {
    fn list_hashes_dyn(&self, f: &mut dyn FnMut(Hash)) {
        self.list_hashes(f);
    }
}

pub trait PointVisitorDyn {
    fn visit_dyn(&mut self, point: Arc<dyn SingularFetch<T = Arc<dyn TraversibleDyn>>>);
}

pub trait TopologicalDyn: ListHashesDyn {
    fn traverse_dyn(&self, visitor: &mut dyn PointVisitorDyn);
}

pub trait TraversibleDyn: Send + Sync + ToOutputDyn + TopologicalDyn + AsAny {}

impl<T: ?Sized + Send + Sync + ToOutputDyn + TopologicalDyn + AsAny> TraversibleDyn for T {}

impl ToOutput for dyn TraversibleDyn {
    fn to_output(&self, output: &mut (impl ?Sized + Output)) {
        self.to_output_dyn(&mut &mut *output);
    }
}

impl Tagged for dyn TraversibleDyn {}

impl ListHashes for dyn TraversibleDyn {
    fn list_hashes(&self, f: &mut (impl ?Sized + FnMut(Hash))) {
        self.list_hashes_dyn(&mut &mut *f);
    }

    fn topology_hash(&self) -> Hash {
        object_rainbow::Hashes(self).data_hash()
    }

    fn point_count(&self) -> usize {
        let mut count = 0;
        self.list_hashes(&mut |_| count += 1);
        count
    }
}

impl<T: ?Sized + PointVisitor> PointVisitorDyn for T {
    fn visit_dyn(&mut self, point: Arc<dyn SingularFetch<T = Arc<dyn TraversibleDyn>>>) {
        self.visit(&point);
    }
}

impl Topological for dyn TraversibleDyn {
    fn traverse(&self, visitor: &mut (impl ?Sized + object_rainbow::PointVisitor)) {
        self.traverse_dyn(&mut &mut *visitor);
    }
}

struct FetchDyn<T>(T);

impl<T: FetchBytes> FetchBytes for FetchDyn<T> {
    fn fetch_bytes(&'_ self) -> object_rainbow::FailFuture<'_, object_rainbow::ByteNode> {
        self.0.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> object_rainbow::FailFuture<'_, Vec<u8>> {
        self.0.fetch_data()
    }
}

impl<T: Singular> Singular for FetchDyn<T> {
    fn hash(&self) -> Hash {
        self.0.hash()
    }
}

impl<T: Fetch<T: Traversible>> Fetch for FetchDyn<T> {
    type T = Arc<dyn TraversibleDyn>;

    fn fetch(&'_ self) -> object_rainbow::FailFuture<'_, Self::T> {
        Box::pin(async move { Ok(Arc::new(self.0.fetch().await?) as _) })
    }
}

impl PointVisitor for dyn '_ + PointVisitorDyn {
    fn visit(&mut self, point: &(impl 'static + SingularFetch<T: Traversible> + Clone)) {
        self.visit_dyn(Arc::new(FetchDyn(point.clone())));
    }
}

impl<T: Traversible> TopologicalDyn for T {
    fn traverse_dyn(&self, visitor: &mut dyn PointVisitorDyn) {
        self.traverse(visitor);
    }
}

pub struct ParsedOpaque(pub Arc<dyn TraversibleDyn>);

#[derive(ToOutput, Tagged, ListHashes, Topological, Clone)]
pub struct Opaque(pub Arc<dyn TraversibleDyn>);

impl<I: PointInput<Extra = Arc<dyn Send + Sync + ExtraFor<ParsedOpaque>>>> Parse<I> for Opaque {
    fn parse(input: I) -> object_rainbow::Result<Self> {
        let extra = input.extra().clone();
        let resolve = input.resolve().clone();
        Ok(Self((*extra).parse(&input.parse_all()?, &resolve)?.0))
    }
}

pub struct OpaqueFactory<Extra, T> {
    pub extra: Extra,
    pub object: ObjectMarker<T>,
}

impl<Extra: Clone, T> Clone for OpaqueFactory<Extra, T> {
    fn clone(&self) -> Self {
        Self {
            extra: self.extra.clone(),
            object: self.object,
        }
    }
}

impl<
    Extra: Clone + ExtraFor<T>,
    T: Send + Sync + Traversible,
    I: PointInput<Extra = OpaqueFactory<Extra, T>>,
> Parse<I> for ParsedOpaque
{
    fn parse(input: I) -> object_rainbow::Result<Self> {
        let extra = input.extra().clone();
        let resolve = input.resolve().clone();
        Ok(Self(Arc::new(
            extra.extra.parse(&input.parse_all()?, &resolve)?,
        )))
    }
}
