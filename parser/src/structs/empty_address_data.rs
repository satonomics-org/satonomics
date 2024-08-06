use allocative::Allocative;
use sanakirja::{direct_repr, Storable, UnsizedStorable};
use serde::{Deserialize, Serialize};

use super::{AddressData, AddressType, Amount};

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Allocative, Serialize, Deserialize,
)]
pub struct EmptyAddressData {
    pub address_type: AddressType,
    pub transfered: Amount,
}
direct_repr!(EmptyAddressData);

impl EmptyAddressData {
    pub fn from_non_empty(non_empty: &AddressData) -> Self {
        if non_empty.sent != non_empty.received {
            dbg!(&non_empty);
            panic!("Trying to convert not empty wallet to empty !");
        }

        Self {
            address_type: non_empty.address_type,
            transfered: non_empty.sent,
        }
    }
}
