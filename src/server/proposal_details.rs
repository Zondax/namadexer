use crate::error::Error;

use serde::{Deserialize, Serialize};
use tracing::info;

use serde_json::Value;
use sqlx::postgres::PgRow as Row;
use sqlx::Row as TRow;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ProposalDetails {
    pub id: i32,
    pub content: Value, // we use serde_json::Value because content is set arbitrarly from the users
    pub r#type: String,
    pub author: String,
    pub voting_start_epoch: i32,
    pub voting_end_epoch: i32,
    pub grace_epoch: i32,
    pub votes: Option<Vec<String>>,
}

impl ProposalDetails {
    fn set_votes(&mut self, votes: Vec<String>) {
        self.votes = Some(votes);
    }

    pub fn add_votes(&mut self, votes: Vec<String>) -> Result<(), Error> {
        self.set_votes(votes);

        return Ok(());
    }
}

impl TryFrom<Row> for ProposalDetails {
    type Error = Error;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        info!("ProposalDetails::try_from");

        let id: i32 = row.try_get("id")?;
        let content: Value = row.try_get("content")?;

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
            votes: None,
        })
    }
}
