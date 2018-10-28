//! Module for configuration access and persistence.

use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait ConfigManage {
    type Data: Serialize + DeserializeOwned;
}
