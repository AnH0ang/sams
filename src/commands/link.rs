use anyhow::Result;

use crate::args::LinkArgs;

pub fn link(_args: LinkArgs) -> Result<()> {
    println!("Linking..");
    Ok(())
}
