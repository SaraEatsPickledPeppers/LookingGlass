//! Two-player co-op. The host owns gameplay decisions; guests supply input.
//!
//! This module is being integrated on the coop branch. Its policy tests are not
//! evidence that the single-player viewer or the campaign already supports co-op.
pub mod check;
pub mod invite;
pub mod rules;
pub mod transport;
pub mod tunnel;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum PlayerId {
    Blue,
    Red,
}
impl PlayerId {
    pub const ALL: [Self; 2] = [Self::Blue, Self::Red];

    pub fn index(self) -> usize {
        match self {
            Self::Blue => 0,
            Self::Red => 1,
        }
    }

    pub fn partner(self) -> Self {
        match self {
            Self::Blue => Self::Red,
            Self::Red => Self::Blue,
        }
    }
}
