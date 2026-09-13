use object_rainbow::{Hash, ListHashes, Output, ToOutput};

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

pub trait PointVisitorDyn {}

pub trait TopologicalDyn {}

pub trait TraversibleDyn: ToOutputDyn {}
