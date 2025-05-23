mod error;
use std::collections::BTreeMap;

pub use error::Error;

mod addon;
pub use addon::Addon;

mod git;
pub use git::Git;

mod manager;
pub use manager::Manager;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Workspace {
    #[serde(default, skip_serializing_if="BTreeMap::is_empty")]
    addons: BTreeMap<String, Addon>
}

pub type LuaRc = luarc::LuaRc<(), (), (), (), (), (), (), (), (), (), (), (), (), Workspace, ()>;
