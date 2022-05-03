use std::env::temp_dir;
use std::fs::{create_dir, File};
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(unix)]
use std::fs::Permissions;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use anyhow::{anyhow, Result};
use log::{info, warn};

/// the lock filename for health check
const LOCK_FILE_NAME: &str = ".lock";

/// now if the server accept connections
static ACCEPTING_CONNECTIONS: AtomicBool = AtomicBool::new(false);

/// check the lockfile if file present or not
#[inline(always)]
pub(crate) fn lock_file_present() -> bool {
    temp_dir().join(LOCK_FILE_NAME).is_file()
}

fn create_lock_file() -> Result<()> {
    if !temp_dir().exists() {
        create_dir(temp_dir())?;
    }

    let path = temp_dir().join(LOCK_FILE_NAME);
    info!("Writing lock-file to: {}", path.display());
    let file = File::create(path)?;
    file.set_len(0)?;

    #[cfg(unix)]
    file.set_permissions(Permissions::from_mode(0660))?;

    Ok(())
}

pub(crate) fn mark_healthy(suppress_lock: bool) -> Result<()> {
    ACCEPTING_CONNECTIONS.store(true, Ordering::Release);

    return if suppress_lock {
        warn!("Warning: \"suppress_lock\" is enabled. No automated health-checks will be in place for your function.");
        Ok(())
    } else {
        match create_lock_file() {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!(
                "Cannot write {}. To disable lock-file set env suppress_lock=true.\n\
                 Error: {}.\n",
                temp_dir().join(LOCK_FILE_NAME).display(),
                e.to_string()
            )),
        }
    };
}

#[inline(always)]
pub(crate) fn check_healthy() -> bool {
    ACCEPTING_CONNECTIONS.load(Ordering::Acquire) || lock_file_present()
}

pub(crate) fn mark_unhealthy() -> Result<(), std::io::Error> {
    ACCEPTING_CONNECTIONS.store(false, Ordering::Release);

    std::fs::remove_file(temp_dir().join(LOCK_FILE_NAME))
}
