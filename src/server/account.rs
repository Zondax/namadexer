use serde::{Deserialize, Serialize};

/// The relevant information regarding accounts.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct AccountUpdates {
    /// Address that identifies this account
    pub account_id: String,
    // The list of vp_code_hashes that this account
    // has been updated with, being the last element in
    // the list the current code_hash this account uses.
    pub code_hashes: Vec<String>,

    /// The list of thresholds that have been configured to
    /// this account. Similar to code hash, the last element
    /// is the threshold being used by this account.
    pub thresholds: Vec<u8>,

    /// The list of public_keys sets that this accounts uses.
    /// Similar to code hash, the last element
    /// is contains the set of public keys this account is associated with.
    pub public_keys: Vec<Vec<String>>,
}
