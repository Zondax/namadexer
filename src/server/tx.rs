use crate::error::Error;
use borsh::BorshDeserialize;
use namada::types::address::Address;
use namada::types::key::common::PublicKey;
use namada::types::{
    token,
    transaction::{
        account::{InitAccount, UpdateAccount},
        governance::VoteProposalData,
        pgf::UpdateStewardCommission,
        pos::{Bond, InitValidator, Unbond, Withdraw},
    },
};

use prost::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::utils::serialize_optional_hex;
use namada::ibc_proto::google::protobuf::Any;
use namada::types::eth_bridge_pool::PendingTransfer;

use namada::ibc::applications::transfer::msgs::transfer::MsgTransfer;

use sqlx::postgres::PgRow as Row;
use sqlx::Row as TRow;

// represents the number of columns
// that db must contains in order to deserialized
// transactions
const NUM_TX_COLUMNS: usize = 9;

// namada::ibc::applications::transfer::msgs::transfer::TYPE_URL has been made private and can't be access anymore
const MSG_TRANSFER_TYPE_URL: &str = "/ibc.applications.transfer.v1.MsgTransfer";

/// Transaction types
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TxDecoded {
    Transfer(token::Transfer),
    Bond(Bond),
    RevealPK(PublicKey),
    VoteProposal(VoteProposalData),
    InitValidator(Box<InitValidator>),
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
    fee_amount_per_gas_unit: String,
    fee_token: String,
    /// Gas limit (only for Wrapper tx)
    gas_limit_multiplier: i64,
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
            // decode tx and update variable
            let unknown_type = "unknown".to_string();
            let type_tx = checksums.get(&self.code()).unwrap_or(&unknown_type);

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
                "tx_init_validator" => InitValidator::try_from_slice(&self.data())
                    .map(|t| TxDecoded::InitValidator(Box::new(t)))?,
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
        if row.len() != NUM_TX_COLUMNS {
            return Err(Error::InvalidTxData);
        }

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
