use crate::error::Error;

use serde::{Deserialize, Serialize};
use tracing::info;

use sqlx::postgres::PgRow as Row;
use sqlx::Row as TRow;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ProposalDetails {
    pub id: i32,
    pub content: Option<String>,
    pub r#type: String,
    pub author: String,
    pub voting_start_epoch: i32,
    pub voting_end_epoch: i32,
    pub grace_epoch: i32,
}

impl TryFrom<Row> for ProposalDetails {
    type Error = Error;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        info!("ProposalDetails::try_from");

        let id: i32 = row.try_get("id")?;
        let content = row.try_get("content")?;
        let r#type = row.try_get("type")?;
        let author = row.try_get("author")?;
        let voting_start_epoch: i32 = row.try_get("voting_start_epoch")?;
        let voting_end_epoch: i32 = row.try_get("voting_end_epoch")?;
        let grace_epoch: i32 = row.try_get("grace_epoch")?;

        Ok(Self {
            id,
            content,
            r#type,
            author,
            voting_start_epoch,
            voting_end_epoch,
            grace_epoch,
        })
    }
}
