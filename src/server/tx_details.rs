use crate::error::Error;
use namada_sdk::ibc::apps::transfer::types::msgs::transfer::MsgTransfer;
use namada_sdk::tx::data::pos::BecomeValidator;
use namada_sdk::types::key::common::PublicKey;
use namada_sdk::{
    account::{InitAccount, UpdateAccount},
    borsh::BorshDeserialize,
    governance::VoteProposalData,
    tx::data::{
        pgf::UpdateStewardCommission,
        pos::{Bond, Unbond, Withdraw},
    },
    types::token,
    types::{address::Address, eth_bridge_pool::PendingTransfer},
};

use namada_sdk::ibc::primitives::proto::Any;
use prost::Message;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

use super::utils::serialize_optional_hex;

use sqlx::postgres::PgRow as Row;
use sqlx::Row as TRow;

use crate::server::tx::{IbcTx, TxDecoded};
use tendermint::Time;
use tendermint::block::Height;

// namada::ibc::applications::transfer::msgs::transfer::TYPE_URL has been made private and can't be access anymore
const MSG_TRANSFER_TYPE_URL: &str = "/ibc.applications.transfer.v1.MsgTransfer";


/// The relevant information regarding transactions and their types.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TxDetails {
    /// The block height of this transaction
    header_height: Height,
    /// The block this transaction belongs to.
    header_time: Time,
    /// The transaction type encoded as a string
    tx_hash: String,
    block_hash: String,
    wrapper_hash: String,
    tx_type: String,
    fee_amount_per_gas_unit: Option<String>,
    fee_token: Option<String>,
    /// Gas limit (only for Wrapper tx)
    gas_limit_multiplier: Option<i64>,
    code_type: Option<String>,

    /// The transaction code. Match what is in the checksum.js
    #[serde(serialize_with = "serialize_optional_hex")]
    code: Option<Vec<u8>>,
    #[serde(serialize_with = "serialize_optional_hex")]
    data: Option<Vec<u8>>,
    memo: String,
    /// Inner transaction type
    tx: Option<TxDecoded>,

    return_code: Option<i32>,
}

impl TxDetails {
    pub fn is_decrypted(&self) -> bool {
        if self.tx_type == "Decrypted" {
            return true;
        }
        false
    }

    pub fn code(&self) -> String {
        let code = self.code.as_deref().unwrap_or_default();
        hex::encode(code)
    }

    pub fn data(&self) -> Vec<u8> {
        self.data.clone().unwrap_or_default()
    }

    fn set_tx(&mut self, tx_decoded: TxDecoded) {
        self.tx = Some(tx_decoded);
    }

    pub fn decode_tx(&mut self, checksums: &HashMap<String, String>) -> Result<(), Error> {
        if self.is_decrypted() {
            let Some(type_tx) = checksums.get(&self.code()) else {
                return Err(Error::InvalidTxData);
            };

            let decoded = match type_tx.as_str() {
                "tx_transfer" => {
                    token::Transfer::try_from_slice(&self.data()).map(TxDecoded::Transfer)?
                }
                "tx_bond" => Bond::try_from_slice(&self.data()).map(TxDecoded::Bond)?,
                "tx_reveal_pk" => {
                    PublicKey::try_from_slice(&self.data()).map(TxDecoded::RevealPK)?
                }
                "tx_vote_proposal" => {
                    VoteProposalData::try_from_slice(&self.data()).map(TxDecoded::VoteProposal)?
                }
                "tx_init_validator" => BecomeValidator::try_from_slice(&self.data())
                    .map(|t| TxDecoded::BecomeValidator(Box::new(t)))?,
                "tx_unbond" => Unbond::try_from_slice(&self.data()).map(TxDecoded::Unbond)?,
                "tx_withdraw" => Withdraw::try_from_slice(&self.data()).map(TxDecoded::Withdraw)?,
                "tx_init_account" => {
                    InitAccount::try_from_slice(&self.data()).map(TxDecoded::InitAccount)?
                }
                "tx_update_account" => {
                    // we could need to give users more context here on how the related accound
                    // has been updated.
                    UpdateAccount::try_from_slice(&self.data()).map(TxDecoded::UpdateAccount)?
                }
                "tx_resign_steward" => {
                    Address::try_from_slice(&self.data()).map(TxDecoded::ResignSteward)?
                }
                "tx_update_steward_commission" => {
                    // we could need to give users more context about this update.
                    UpdateStewardCommission::try_from_slice(&self.data())
                        .map(TxDecoded::UpdateStewardCommission)?
                }
                "tx_ibc" => Self::decode_ibc(&self.data()).map(TxDecoded::Ibc)?,
                "tx_bridge_pool" => {
                    PendingTransfer::try_from_slice(&self.data()).map(TxDecoded::EthPoolBridge)?
                }
                _ => {
                    return Err(Error::InvalidTxData);
                }
            };

            self.set_tx(decoded);

            return Ok(());
        }
        Err(Error::InvalidTxData)
    }

    fn decode_ibc(tx_data: &[u8]) -> Result<IbcTx, Error> {
        let msg = Any::decode(tx_data).map_err(|_| Error::InvalidTxData)?;
        if msg.type_url.as_str() == MSG_TRANSFER_TYPE_URL
            && MsgTransfer::try_from(msg.clone()).is_ok()
        {
            Ok(IbcTx::MsgTransfer(msg))
        } else {
            Ok(IbcTx::Any(msg))
        }
    }
}

impl TryFrom<Row> for TxDetails {
    type Error = Error;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        info!("TxDetails::try_from");

        // height
        let header_height: i32 = row.try_get("header_height")?;
        let header_height = Height::from(header_height as u32);
        
        // timestamp
        let timestamp: &str = row.try_get("header_time")?;
        let header_time = Time::parse_from_rfc3339(timestamp)?;
        
        let tx_hash: String = row.try_get("tx_hash")?;
        let block_hash: String = row.try_get("block_hash")?;
        let wrapper_hash: String = row.try_get("wrapper_hash")?;
        let tx_type: String = row.try_get("tx_type")?;

        let fee_amount_per_gas_unit = row.try_get("fee_amount_per_gas_unit")?;
        let fee_token = row.try_get("fee_token")?;
        let gas_limit_multiplier = row.try_get("gas_limit_multiplier")?;
        let code_type = row.try_get("code_type")?;
        let code: Option<Vec<u8>> = row.try_get("code")?;
        let data: Option<Vec<u8>> = row.try_get("data")?;
        let memo: String = row.try_get("memo")?;
        let return_code: Option<i32> = row.try_get("return_code")?;

        Ok(Self {
            header_height,
            header_time,
            tx_hash,
            block_hash,
            wrapper_hash,
            tx_type,
            fee_amount_per_gas_unit,
            fee_token,
            gas_limit_multiplier,
            code_type,
            code,
            data,
            memo,
            tx: None,
            return_code
        })
    }
}
