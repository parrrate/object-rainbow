use crate::*;

impl ToOutput for str {
    fn to_output(&self, output: &mut dyn Output) {
        if output.is_real() {
            output.write(self.as_bytes());
        }
    }
}

impl ListHashes for str {}
impl Topological for str {}
impl Tagged for str {}
