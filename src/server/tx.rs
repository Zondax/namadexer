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

// namada::ibc::applications::transfer::msgs::transfer::TYPE_URL has been made private and can't be access anymore
const MSG_TRANSFER_TYPE_URL: &str = "/ibc.applications.transfer.v1.MsgTransfer";

/// Transaction types
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TxDecoded {
    Transfer(token::Transfer),
    Bond(Bond),
    RevealPK(PublicKey),
    VoteProposal(VoteProposalData),
    BecomeValidator(Box<BecomeValidator>),
    Unbond(Unbond),
    Withdraw(Withdraw),
    InitAccount(InitAccount),
    UpdateAccount(UpdateAccount),
    ResignSteward(Address),
    UpdateStewardCommission(UpdateStewardCommission),
    EthPoolBridge(PendingTransfer),
    Ibc(IbcTx),
}

// we have a variant for MsgTransfer, but there are other message types
// defined in https://github.com/cosmos/ibc-rs/blob/main/crates/ibc/src/core/msgs.rs
// however none of them implement serde traits, so lets use Any as the general
// abstraction for it. Any is serializeable
/// Defines the support IBC transactions.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum IbcTx {
    // we do not use MsgTransfer directly as it does not
    // implements serde traits.
    MsgTransfer(Any),
    Any(Any),
}

/// The relevant information regarding transactions and their types.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TxInfo {
    /// The hash that idenfities this transaction
    #[serde(with = "hex::serde")]
    hash: Vec<u8>,
    /// The block this transaction belongs to.
    #[serde(with = "hex::serde")]
    block_id: Vec<u8>,
    /// The transaction type encoded as a string
    tx_type: String,
    /// id for the wrapper tx if the tx is decrypted. otherwise it is null.
    #[serde(with = "hex::serde")]
    wrapper_id: Vec<u8>,
    /// The transaction fee only for tx_type Wrapper (otherwise empty)
    fee_amount_per_gas_unit: Option<String>,
    fee_token: Option<String>,
    /// Gas limit (only for Wrapper tx)
    gas_limit_multiplier: Option<i64>,
    /// The transaction code. Match what is in the checksum.js
    #[serde(serialize_with = "serialize_optional_hex")]
    code: Option<Vec<u8>>,
    #[serde(serialize_with = "serialize_optional_hex")]
    data: Option<Vec<u8>>,
    /// Inner transaction type
    tx: Option<TxDecoded>,
}

impl TxInfo {
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

impl TryFrom<Row> for TxInfo {
    type Error = Error;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        info!("TxInfo::try_from");

        let hash: Vec<u8> = row.try_get("hash")?;
        let block_id: Vec<u8> = row.try_get("block_id")?;
        let tx_type: String = row.try_get("tx_type")?;
        let wrapper_id: Vec<u8> = row.try_get("wrapper_id")?;
        let fee_amount_per_gas_unit = row.try_get("fee_amount_per_gas_unit")?;
        let fee_token = row.try_get("fee_token")?;
        let gas_limit_multiplier = row.try_get("gas_limit_multiplier")?;
        let code: Option<Vec<u8>> = row.try_get("code")?;
        let data: Option<Vec<u8>> = row.try_get("data")?;

        Ok(Self {
            hash,
            block_id,
            tx_type,
            wrapper_id,
            fee_amount_per_gas_unit,
            fee_token,
            gas_limit_multiplier,
            code,
            data,
            tx: None,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct VoteProposalTx {
    pub id: u64,
    pub vote: String,
    pub voter: String,
    pub delegations: Vec<String>,
    #[serde(with = "hex::serde")]
    pub tx_id: Vec<u8>,
}

impl TryFrom<Row> for VoteProposalTx {
    type Error = Error;

    fn try_from(value: Row) -> Result<Self, Self::Error> {
        let id = value.try_get::<[u8; std::mem::size_of::<u64>()], _>("vote_proposal_id")?;
        let id = u64::from_be_bytes(id);

        let vote = value.try_get::<String, _>("vote")?;
        let voter = value.try_get::<String, _>("voter")?;
        let tx_id = value.try_get::<Vec<u8>, _>("tx_id")?;

        // empty this comes from another table.
        let delegations = vec![];

        Ok(Self {
            id,
            vote,
            voter,
            delegations,
            tx_id,
        })
    }
}
