use crate::*;

impl ToOutput for char {
    fn to_output(&self, output: &mut dyn Output) {
        let mut buf = [0; 4];
        self.encode_utf8(&mut buf).to_output(output);
    }
}

impl InlineOutput for char {}
impl Tagged for char {}
impl ListHashes for char {}
impl Topological for char {}
