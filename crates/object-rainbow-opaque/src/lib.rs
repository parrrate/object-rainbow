use std::sync::Arc;

use object_rainbow::{
    ExtraFor, Fetch, FetchBytes, Hash, ListHashes, Output, Parse, PointInput, PointVisitor,
    Singular, SingularFetch, Tagged, ToOutput, Topological, Traversible,
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

pub trait TraversibleDyn: Send + Sync + ToOutputDyn + TopologicalDyn {}

impl<T: ?Sized + Send + Sync + ToOutputDyn + TopologicalDyn> TraversibleDyn for T {}

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

#[derive(ToOutput)]
pub struct Opaque(pub Arc<dyn TraversibleDyn>);

impl<I: PointInput<Extra = Arc<dyn Send + Sync + ExtraFor<Arc<dyn TraversibleDyn>>>>> Parse<I>
    for Opaque
{
    fn parse(input: I) -> object_rainbow::Result<Self> {
        let extra = input.extra().clone();
        let resolve = input.resolve().clone();
        extra.parse(&input.parse_all()?, &resolve)
    }
}
