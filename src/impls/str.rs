use crate::*;

impl ToOutput for str {
    fn to_output(&self, output: &mut dyn Output) {
        output.write(self.as_bytes());
    }
}

impl Topological for str {}
impl Tagged for str {}
