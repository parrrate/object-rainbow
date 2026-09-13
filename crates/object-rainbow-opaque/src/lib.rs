use std::sync::Arc;

use object_rainbow::{Hash, ListHashes, Output, SingularFetch, ToOutput};

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
