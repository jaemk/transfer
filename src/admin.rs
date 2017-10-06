/*!
General Admin commands
*/
use std::env;

use sweep;
use errors::*;


/// Cleanup files that no longer have an associated record in the database
pub fn sweep_files() -> Result<()> {
    let dir = env::current_dir()?;
    let upload_dir = dir.join("uploads");
    if upload_dir.is_dir() && upload_dir.exists() {
        let n = sweep::sweep_fs(&upload_dir)?;
        info!("** Cleaned up {} orphaned files **", n);
    } else {
        bail!("Provided upload dir is invalid: {:?}", upload_dir);
    }
    Ok(())
}

