use object_rainbow::Output;

pub trait ToOutputDyn {
    fn to_output_dyn(&self, output: &mut dyn Output);
}
