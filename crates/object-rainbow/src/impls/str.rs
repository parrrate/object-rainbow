use crate::*;

impl ToOutput for str {
    fn to_output(&self, output: &mut impl Output) {
        self.as_bytes().to_output(output);
    }
}

impl ListHashes for str {}
impl Topological for str {}
impl Tagged for str {}
impl MaybeHasNiche for str {
    type MnArray = NoNiche<NicheForUnsized>;
}

impl ByteOrd for str {
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}
