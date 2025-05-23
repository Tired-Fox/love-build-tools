use std::{collections::BTreeMap, path::PathBuf};

use luarc::{Error, LuaRc};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Target {
    #[default]
    LuaCats,
    Github,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Addon {
    pub src: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    pub target: Target,
}

/**
    Default Workspace

    ```json
    {
        "workspace": {
            "checkThirdParty": false,
            "ignore_dir": [],
            "ignoreSubmodules": true,
            "library": [],
            "maxPreload": 5000,
            "preloadFileSize": 500,
            "useGitIgnore": true,
            "user_third_party": [],
        }
    }
    ```

    Extended Workspace

    ```json
    {
        "workspace": {
            ...
            addons: {}
        }
    }
    ```
*/
#[derive(Debug, Serialize, Deserialize)]
struct Workspace {
    #[serde(default, skip_serializing_if="BTreeMap::is_empty")]
    addons: BTreeMap<String, Addon>
}

fn main() -> Result<(), Error> {
    /*
        This will extend specific parts of the configuration by setting generic types in LuaRc and it's children.
        It makes use of the `serde` `flatten` attribute meaning all values in the provided struct are on the same
        level as the parent. This lets the user extend the base configuration with their own custom fields and types.

        For convenience LuaRc and it's subtypes implement `Deref` and `DerefMut` for the type provided. This means you can
        access the fields on the parent as you would expect when extending (adding more fields) the parent type.

        Use `root` to extend at the base level of the configuration object. Then use the name of the subsection to extend 
        that sections object with your custom type.
    */

    let rc = LuaRc::extend()
        // All the fields found in the `Workspace` type will now be parsed as if they are at the same level
        // as the default Workspace's fields
        .workspace::<Workspace>()
        .read(PathBuf::from("examples/customrc.json"))?
        .unwrap_or_default();

    if let Some(workspace) = rc.workspace.as_ref() { 
        println!("{workspace:#?}");
    } else {
        println!("No workspace in configuration");
    }

    Ok(())
}
