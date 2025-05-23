use std::path::PathBuf;

use luarc::{Error, LuaRc};

fn main() -> Result<(), Error> {
    let rc = LuaRc::read(PathBuf::from("examples/basicrc.json"))?.unwrap_or_default();
    println!("{:#?}", rc);

    Ok(())
}
