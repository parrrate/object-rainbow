use crate::map_extra::MapExtra;

pub struct WithoutHeader;

impl<H: 'static + Clone, E: 'static + Clone> MapExtra<(H, E)> for WithoutHeader {
    type Mapped = E;

    fn map_extra(&self, (_, extra): (H, E)) -> Self::Mapped {
        extra
    }
}
