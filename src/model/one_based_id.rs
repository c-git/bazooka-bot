use std::{fmt::Display, num::NonZeroUsize};

/// Stores an ID. These are 1 based because that's
/// how it gets displayed because of markdown
#[derive(
    Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy,
)]
pub struct OneBasedId(NonZeroUsize);

impl OneBasedId {
    pub fn as_index(&self) -> usize {
        let x: usize = self.0.into();
        x - 1 // Convert back down to 0 based
    }
    pub fn from_index(value: usize) -> Self {
        let x = NonZeroUsize::new(value + 1).expect("any usize plus 1 must be non-zero");
        Self(x)
    }
}

impl From<NonZeroUsize> for OneBasedId {
    fn from(value: NonZeroUsize) -> Self {
        Self(value)
    }
}

impl Display for OneBasedId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
