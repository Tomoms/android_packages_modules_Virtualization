/*
 * Copyright (C) 2021 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use anyhow::{bail, Result};
use nix::sys::stat::FileStat;
use std::fs;
use std::fs::File;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

/// Returns when the file exists on the given `path` or timeout (1s) occurs.
pub fn wait_for_path<P: AsRef<Path>>(path: P) -> Result<()> {
    const TIMEOUT: Duration = Duration::from_secs(1);
    const INTERVAL: Duration = Duration::from_millis(10);
    let begin = Instant::now();
    while !path.as_ref().exists() {
        if begin.elapsed() > TIMEOUT {
            bail!("{:?} not found. TIMEOUT.", path.as_ref());
        }
        thread::sleep(INTERVAL);
    }
    Ok(())
}

/// Returns hexadecimal reprentation of a given byte array.
pub fn hexstring_from(s: &[u8]) -> String {
    s.iter().map(|byte| format!("{:02x}", byte)).reduce(|i, j| i + &j).unwrap_or(String::new())
}

/// fstat that accepts a path rather than FD
pub fn fstat(p: &Path) -> Result<FileStat> {
    let f = File::open(p)?;
    Ok(nix::sys::stat::fstat(f.as_raw_fd())?)
}

// From include/uapi/linux/fs.h
const BLK: u8 = 0x12;
const BLKGETSIZE64: u8 = 114;
nix::ioctl_read!(_blkgetsize64, BLK, BLKGETSIZE64, libc::size_t);

/// Gets the size of a block device
pub fn blkgetsize64(p: &Path) -> Result<u64> {
    let f = File::open(p)?;
    if !f.metadata()?.file_type().is_block_device() {
        bail!("{:?} is not a block device", p);
    }
    let mut size: usize = 0;
    // SAFETY: kernel copies the return value out to `size`. The file is kept open until the end of
    // this function.
    unsafe { _blkgetsize64(f.as_raw_fd(), &mut size) }?;
    Ok(size as u64)
}
