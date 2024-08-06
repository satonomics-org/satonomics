use allocative::Allocative;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

// https://unchained.com/blog/bitcoin-address-types-compared/
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    Allocative,
)]
pub enum AddressType {
    Empty,
    OpReturn,
    PushOnly,
    #[default]
    Unknown,
    MultiSig,
    P2PK,
    P2PKH,
    P2SH,
    P2WPKH,
    P2WSH,
    P2TR,
}
