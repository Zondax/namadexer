use crate::database::Database;
use crate::error::Error;
use sqlx::Row as TRow;
use tracing::instrument;

#[instrument(name = "Utils::get_start_height", skip(db))]
pub async fn get_start_height(db: &Database) -> Result<u32, Error> {
    let last_row = db.get_last_height().await?;

    // height must be greater than 0 so we start at 1
    let mut current_height: i32 = last_row.try_get("header_height").unwrap_or(0);
    current_height += 1;

    Ok(current_height as u32)
}

#[instrument(name = "Utils::has_indexes", skip(db))]
pub async fn has_indexes(db: &Database) -> Result<bool, Error> {
    let indexes_row = db.check_indexes().await?;

    let mut has_indexes = false;
    if !indexes_row.is_empty() {
        tracing::info!("We already have indexes created resuming from last block indexed");
        has_indexes = true;
    }

    Ok(has_indexes)
}
