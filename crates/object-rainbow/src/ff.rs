use typenum::{U1, U254};

use crate::{decr_byte_niche::DecrByteNiche, *};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    ListHashes,
    Topological,
    ParseAsInline,
    Default,
)]
pub struct Ff;

impl ToOutput for Ff {
    fn to_output(&self, output: &mut impl Output) {
        output.write(&[0xff]);
    }
}

impl InlineOutput for Ff {}

impl<I: ParseInput> ParseInline<I> for Ff {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        if input.parse_inline::<u8>()? == 0xff {
            Ok(Self)
        } else {
            Err(Error::OutOfBounds)
        }
    }
}

impl ByteOrd for Ff {
    fn bytes_cmp(&self, Self: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl MaybeHasNiche for Ff {
    type MnArray = SomeNiche<DecrByteNiche<U254>>;
}

#[test]
fn ff_option() {
    assert_eq!(Some(Ff).vec(), [0xff]);
    assert_eq!(None::<Ff>.vec(), [0xfe]);
    assert_eq!(None::<Option<Ff>>.vec(), [0xfd]);
    assert_eq!(None::<Option<Option<Ff>>>.vec(), [0xfc]);
}

impl Size for Ff {
    type Size = U1;
}

impl Monostate for Ff {}

#[test]
fn ff_collection() {
    use crate::{monostate_headers::MonostateHeaders, niche_cut::NicheCut, none_terminated::Nt};
    type Stuff = Nt<MonostateHeaders<Vec<()>, (Ff, NicheCut)>>;
    let mut stuff = Stuff::default();
    stuff.push(());
    stuff.push(());
    assert_eq!(Stuff::vec(&stuff), [0xff, 0xff, 0xfe]);
}
