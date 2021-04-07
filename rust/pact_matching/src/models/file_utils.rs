use std::fs::File;
use std::path::Path;
use log::*;
use fs2::FileExt;
use std::thread::sleep;
use std::time::Duration;

use anyhow::bail;

pub(crate) fn with_read_lock_for_open_file<T>(
  path: &Path,
  file: &mut File,
  attempts: u32,
  cl: &mut dyn FnMut(&mut File) -> anyhow::Result<T>
) -> anyhow::Result<T> {
  let mut attempt = 0;
  while attempt < attempts {
   trace!("Attempt {} of {} to get a shared lock on '{:?}'", attempt, attempts, path);
   match file.try_lock_shared() {
     Ok(_) => {
       trace!("Got shared file lock");
       let result = cl(file);
       trace!("Releasing shared file lock");
       if let Err(err) = file.unlock() {
         warn!("Failed to release shared lock on '{}' - {}", path.to_string_lossy(), err);
       }
       return result;
     }
     Err(err) => {
       attempt += 1;
       let s = 100 * 2_u64.pow(attempt);
       trace!("Failed to get file lock, sleeping for {} ms: {}", s, err);
       sleep(Duration::from_millis(s));
     }
   }
  }
  let msg = format!("Could not acquire a shared lock on '{}' after {} attempts",
                    path.to_string_lossy(), attempts);
  error!("{}", msg);
  bail!(msg);
}

pub(crate) fn with_read_lock<T>(
  path: &Path,
  attempts: u32,
  cl: &mut dyn FnMut(&mut File) -> anyhow::Result<T>
) -> anyhow::Result<T> {
  let mut file = File::open(path)?;
  with_read_lock_for_open_file(path, &mut file, attempts, cl)
}

pub(crate) fn with_write_lock<T>(
  path: &Path,
  file: &mut File,
  attempts: u32,
  cl: &mut dyn FnMut(&mut File) -> anyhow::Result<T>
) -> anyhow::Result<T> {
  let mut attempt = 0;
  while attempt < attempts {
    trace!("Attempt {} of {} to get an exclusive lock on '{:?}'", attempt, attempts, path);
    match file.try_lock_exclusive() {
      Ok(_) => {
        trace!("Got exclusive file lock");
        let result = cl(file);
        trace!("Releasing exclusive file lock");
        if let Err(err) = file.unlock() {
          warn!("Failed to release exclusive lock on '{}' - {}", path.to_string_lossy(), err);
        }
        return result;
      }
      Err(err) => {
        attempt += 1;
        let s = 100 * 2_u64.pow(attempt);
        trace!("Failed to get file lock, sleeping for {} ms: {}", s, err);
        sleep(Duration::from_millis(s));
      }
    }
  }
  let msg = format!("Could not acquire an exclusive lock on '{}' after {} attempts",
                    path.to_string_lossy(), attempts);
  error!("{}", msg);
  bail!(msg);
}
