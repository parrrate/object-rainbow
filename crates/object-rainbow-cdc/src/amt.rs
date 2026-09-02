use object_rainbow::zero_terminated::Zt;
use object_rainbow_amt::AmtMap;
use object_rainbow_point::Point;

use crate::Chunks;

pub type FileMap = AmtMap<Zt<String>, Option<Point<Chunks>>>;
