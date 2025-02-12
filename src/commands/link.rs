use anyhow::Result;

use crate::args::{GlobalArgs, LinkArgs};

pub fn link(_args: LinkArgs, _global_args: GlobalArgs) -> Result<()> {
    println!("Linking..");
    Ok(())
}
