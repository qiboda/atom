#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IsosurfaceSide {
    Inside,
    Outside,
}

impl From<bool> for IsosurfaceSide {
    fn from(value: bool) -> Self {
        if value {
            IsosurfaceSide::Outside
        } else {
            IsosurfaceSide::Inside
        }
    }
}
