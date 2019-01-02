/*!
General Admin commands
*/
use sweep;
use models::CONFIG;
use error::{self, Result};


/// Cleanup files that no longer have an associated record in the database
pub fn sweep_files() -> Result<()> {
    let upload_dir = CONFIG.upload_dir()?;
    if upload_dir.is_dir() && upload_dir.exists() {
        let n = sweep::sweep_fs(&upload_dir)?;
        info!("** Cleaned up {} orphaned files **", n);
    } else {
        return Err(error::helpers::internal(format!("Provided upload dir is invalid: {:?}", upload_dir)));
    }
    Ok(())
}

