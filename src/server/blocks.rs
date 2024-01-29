use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{instrument, trace};

use sqlx::postgres::PgRow as Row;
use sqlx::Row as TRow;
use tendermint::block::{Height, Round};
use tendermint::AppHash;
use tendermint::{
    account::Id as AccountId,
    block::header::Version,
    block::{parts::Header as PartSetHeader, Header, Id as BlockId},
    chain::Id,
    Hash, Time,
};

use super::{from_hex, serialize_hex};
use crate::error::Error;

/// Last commit info in a block
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LastCommitInfo {
    pub height: Height,
    pub round: Round,
    pub block_id: BlockId,
}

impl LastCommitInfo {
    fn read_from(row: &Row) -> Result<Option<Self>, Error> {
        tracing::trace!("Deserializing LastCommitInfo",);

        // height
        let height: Option<i32> = row.try_get("commit_height")?;
        let height = height.map(|h| Height::from(h as u32));
        let Some(height) = height else {
            return Ok(None);
        };

        // round
        let round: Option<i32> = row.try_get("commit_round")?;
        let Some(round) = round else {
            return Ok(None);
        };
        let round = Round::try_from(round as u32)?;

        // BlockId
        let hash: Option<Vec<u8>> = row.try_get("commit_block_id_hash")?;
        let Some(hash) = hash else {
            return Ok(None);
        };

        let hash = Hash::try_from(hash)?;

        // part_set_header
        let total: Option<i32> = row.try_get("commit_block_id_parts_header_total")?;
        let Some(total) = total else {
            return Ok(None);
        };

        let h_hash: Option<Vec<u8>> = row.try_get("commit_block_id_parts_header_hash")?;
        let Some(h_hash) = h_hash else {
            return Ok(None);
        };
        let h_hash = Hash::try_from(h_hash)?;

        let part_set_header = PartSetHeader::new(total as u32, h_hash)?;

        let block_id = BlockId {
            hash,
            part_set_header,
        };

        Ok(Some(Self {
            height,
            round,
            block_id,
        }))
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct HashID(
    #[serde(serialize_with = "serialize_hex", deserialize_with = "from_hex")] pub Vec<u8>,
);

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxShort {
    pub tx_type: String,
    pub hash_id: HashID,
}

/// Relevant information regarding blocks
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BlockInfo {
    pub block_id: HashID,
    pub header: Header,
    pub last_commit: Option<LastCommitInfo>,
    pub tx_hashes: Vec<TxShort>,
}

impl From<BlockInfo> for Header {
    fn from(value: BlockInfo) -> Self {
        value.header
    }
}

impl TryFrom<&Row> for BlockInfo {
    type Error = Error;

    #[instrument(level = "trace", skip(row))]
    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        // block_id
        let block_id: Vec<u8> = row.try_get("block_id")?;
        trace!("parsed block_id {:?}", &block_id);

        // app_version
        let app_version: i32 = row.try_get("header_version_block")?;
        // block_version
        let block_version: i32 = row.try_get("header_version_block")?;

        let version = Version {
            block: block_version as u64,
            app: app_version as u64,
        };
        trace!(
            "parsed block_version: {} - app_version: {}",
            block_version,
            app_version
        );

        // chain_id
        let chain_id: String = row.try_get("header_chain_id")?;
        let chain_id = Id::from_str(&chain_id)?;
        trace!("parsed chain_id {}", &chain_id);

        // height
        let height: i32 = row.try_get("header_height")?;
        let height = Height::from(height as u32);
        trace!("parsed height {}", height);

        // timestamp
        let timestamp: &str = row.try_get("header_time")?;
        trace!("parsed timestamp {}", timestamp);
        let time = Time::parse_from_rfc3339(timestamp)?;

        // parsing Header::Option<last_block_id> ***********************
        let last_block_id = 'block_id: {
            trace!("deserializing last_block_id");
            let hash: Option<Vec<u8>> = row.try_get("header_last_block_id_hash")?;
            let Some(hash) = hash else {
                break 'block_id None;
            };

            let hash = Hash::try_from(hash)?;

            // part_set_header
            let total: Option<i32> = row.try_get("header_last_block_id_parts_header_total")?;
            let Some(total) = total else {
                // if we reach this point, means that there is a non-null
                // last_block_id, so all other fields for block_id should not
                // be null
                return Err(Error::InvalidBlockData);
            };

            let h_hash: Option<Vec<u8>> = row.try_get("header_last_block_id_parts_header_hash")?;
            let Some(h_hash) = h_hash else {
                // if we reach this point, means that there is a non-null
                // last_block_id, so all other fields for block_id should not
                // be null
                return Err(Error::InvalidBlockData);
            };
            let h_hash = Hash::try_from(h_hash)?;

            let part_set_header = PartSetHeader::new(total as u32, h_hash)?;

            Some(BlockId {
                hash,
                part_set_header,
            })
        };
        // ******************************

        // last_commit_hash
        let hash: Option<Vec<u8>> = row.try_get("header_last_commit_hash")?;
        // None
        let last_commit_hash = hash.and_then(|h| Hash::try_from(h).ok());

        // data_hash
        let hash: Option<Vec<u8>> = row.try_get("header_data_hash")?;
        let data_hash = hash.and_then(|h| Hash::try_from(h).ok());

        // validators_hash
        let hash: Vec<u8> = row.try_get("header_next_validators_hash")?;
        let validators_hash = Hash::try_from(hash).map_err(|_| Error::InvalidBlockData)?;

        // next_validators_hash
        let hash: Vec<u8> = row.try_get("header_next_validators_hash")?;
        let next_validators_hash = Hash::try_from(hash).map_err(|_| Error::InvalidBlockData)?;

        // consensus_hash
        let hash: Vec<u8> = row.try_get("header_consensus_hash")?;
        let consensus_hash = Hash::try_from(hash).map_err(|_| Error::InvalidBlockData)?;

        // app_hash
        let hash: &str = row.try_get("header_app_hash")?;
        let app_hash = AppHash::from_str(hash)?;

        // last_results_hash
        let hash: Option<Vec<u8>> = row.try_get("header_last_results_hash")?;
        let last_results_hash = hash.and_then(|h| Hash::try_from(h).ok());

        // evidence_hash
        let hash: Option<Vec<u8>> = row.try_get("header_evidence_hash")?;
        let evidence_hash = hash.and_then(|h| Hash::try_from(h).ok());

        // proposer_address
        let id: &str = row.try_get("header_proposer_address")?;
        let proposer_address = AccountId::from_str(id)?;

        let last_commit = LastCommitInfo::read_from(row)?;

        let header = Header {
            version,
            chain_id,
            height,
            time,
            last_block_id,
            last_commit_hash,
            data_hash,
            validators_hash,
            next_validators_hash,
            consensus_hash,
            app_hash,
            last_results_hash,
            evidence_hash,
            proposer_address,
        };

        Ok(BlockInfo {
            block_id: HashID(block_id),
            header,
            last_commit,
            tx_hashes: vec![],
        })
    }
}
