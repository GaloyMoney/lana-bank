pub mod entity;
pub mod error;
pub(crate) mod publisher;
pub mod repo;

pub use entity::*;
pub use error::*;
pub use repo::{
    ProspectRepo, ProspectsFilters, ProspectsSortBy as RepoProspectsSortBy, Sort, prospect_cursor,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProspectsSortBy {
    CreatedAt,
    Email,
    TelegramHandle,
}

impl From<(ProspectsSortBy, &entity::Prospect)> for prospect_cursor::ProspectsCursor {
    fn from(prospect_with_sort: (ProspectsSortBy, &entity::Prospect)) -> Self {
        let (sort, prospect) = prospect_with_sort;
        match sort {
            ProspectsSortBy::CreatedAt => {
                prospect_cursor::ProspectsByCreatedAtCursor::from(prospect).into()
            }
            ProspectsSortBy::Email | ProspectsSortBy::TelegramHandle => {
                prospect_cursor::ProspectsByIdCursor::from(prospect).into()
            }
        }
    }
}
