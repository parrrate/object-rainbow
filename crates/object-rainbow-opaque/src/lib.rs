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
    fn visit(&mut self, point: Arc<dyn SingularFetch<T = Arc<dyn TraversibleDyn>>>);
}

pub trait TopologicalDyn: ListHashesDyn {}

pub trait TraversibleDyn: Send + Sync + ToOutputDyn + TopologicalDyn {}

impl ToOutput for dyn TraversibleDyn {
    fn to_output(&self, output: &mut (impl ?Sized + Output)) {
        self.to_output_dyn(&mut &mut *output);
    }
}
