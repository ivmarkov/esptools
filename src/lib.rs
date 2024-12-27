//! A module for bundling the Espressif `esptools` binaries with a Rust application.
//! See https://github.com/espressif/esptool/releases

use core::fmt::{self, Display};

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use flate2::read::GzDecoder;

use log::info;

const WINDOWS: bool = cfg!(target_os = "windows");

static MOUNT_POINTS: Mutex<BTreeMap<Tool, PathBuf>> = Mutex::new(BTreeMap::new());

/// ESP tools supported by this crate
#[allow(clippy::enum_variant_names)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Tool {
    /// `esptool.py`
    #[cfg(feature = "esptool")]
    EspTool,
    /// `espsecure.py`
    #[cfg(feature = "espsecure")]
    EspSecure,
    /// `espefuse.py`
    #[cfg(feature = "espefuse")]
    EspEfuse,
    /// `esp-idf-nvs-partition-gen.py`
    #[cfg(feature = "espidfnvs")]
    EspIdfNvs,
}

impl Tool {
    const SHA1: &[&str] = &[
        #[cfg(feature = "esptool")]
        env!("TOOL_ESPTOOL_SHA1"),
        #[cfg(feature = "espsecure")]
        env!("TOOL_ESPSECURE_SHA1"),
        #[cfg(feature = "espefuse")]
        env!("TOOL_ESPEFUSE_SHA1"),
        #[cfg(feature = "espidfnvs")]
        env!("TOOL_ESPIDFNVS_SHA1"),
    ];

    const GZ_BINARY: &[&[u8]] = &[
        #[cfg(feature = "esptool")]
        include_bytes!(env!("TOOL_ESPTOOL")),
        #[cfg(feature = "espsecure")]
        include_bytes!(env!("TOOL_ESPSECURE")),
        #[cfg(feature = "espefuse")]
        include_bytes!(env!("TOOL_ESPEFUSE")),
        #[cfg(feature = "espidfnvs")]
        include_bytes!(env!("TOOL_ESPIDFNVS")),
    ];

    /// Mount a tool
    ///
    /// This ensures that the tool is expanded in a cache directory private to the crate
    /// and is ready for use.
    ///
    /// # Returns
    /// * `Ok(MountedTool)` if the tool was successfully mounted
    /// * `Err(std::io::Error)` if the tool could not be mounted
    pub fn mount(&self) -> Result<MountedTool, io::Error> {
        let mut mount_points = MOUNT_POINTS.lock().unwrap();

        if mount_points.contains_key(self) {
            return Ok(MountedTool(*self));
        }

        let project_dirs = directories::ProjectDirs::from("org", "ivmarkov", "esptools")
            .ok_or(io::ErrorKind::NotFound)?;

        let tools_dir = project_dirs.cache_dir().to_path_buf();

        let path = tools_dir.join(self.sha1()).join(self.name(WINDOWS));

        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap())?;

            let mut read = GzDecoder::new(SliceReader::new(self.gz()));
            let mut write = fs::File::create(&path)?;

            io::copy(&mut read, &mut write)?;

            #[cfg(unix)]
            {
                use std::fs::Permissions;
                use std::os::unix::fs::PermissionsExt;

                std::fs::set_permissions(&path, Permissions::from_mode(0o755))?;
            }
        }

        mount_points.insert(*self, path.clone());

        info!("Tool {self} mounted as `{}`", path.display());

        Ok(MountedTool(*self))
    }

    #[allow(unused)]
    #[allow(unreachable_patterns)]
    pub fn cmd_matches(&self, _cmd: &str) -> bool {
        let _cmd = _cmd.to_ascii_lowercase();
        let _cmd = _cmd.as_str();

        match self {
            #[cfg(feature = "esptool")]
            Self::EspTool => matches!(_cmd, "tool" | "flash"),
            #[cfg(feature = "espsecure")]
            Self::EspSecure => matches!(_cmd, "secure"),
            #[cfg(feature = "espefuse")]
            Self::EspEfuse => matches!(_cmd, "efuse"),
            #[cfg(feature = "espidfnvs")]
            Self::EspIdfNvs => matches!(_cmd, "idfnvs"),
            _ => unreachable!(),
        }
    }

    #[allow(unused)]
    #[allow(unreachable_patterns)]
    pub const fn cmd_description(&self) -> &str {
        match self {
            #[cfg(feature = "esptool")]
            Self::EspTool => "`tool` (or `flash`)",
            #[cfg(feature = "espsecure")]
            Self::EspSecure => "`secure`",
            #[cfg(feature = "espefuse")]
            Self::EspEfuse => "`efuse`",
            #[cfg(feature = "espidfnvs")]
            Self::EspIdfNvs => "`idfnvs`",
            _ => unreachable!(),
        }
    }

    /// Return an iterator over all tools
    #[allow(unused)]
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            #[cfg(feature = "esptool")]
            Self::EspTool,
            #[cfg(feature = "espsecure")]
            Self::EspSecure,
            #[cfg(feature = "espefuse")]
            Self::EspEfuse,
            #[cfg(feature = "espidfnvs")]
            Self::EspIdfNvs,
        ]
        .into_iter()
    }

    /// Return the base file name of the tool
    #[allow(unreachable_patterns)]
    const fn basename(&self) -> &str {
        match self {
            #[cfg(feature = "esptool")]
            Self::EspTool => "esptool",
            #[cfg(feature = "espsecure")]
            Self::EspSecure => "espsecure",
            #[cfg(feature = "espefuse")]
            Self::EspEfuse => "espefuse",
            #[cfg(feature = "espidfnvs")]
            Self::EspIdfNvs => "espidfnvs",
            _ => unreachable!(),
        }
    }

    /// Return the file name of the tool
    #[allow(unreachable_patterns)]
    const fn name(&self, windows: bool) -> &str {
        if windows {
            match self {
                #[cfg(feature = "esptool")]
                Self::EspTool => "esptool.exe",
                #[cfg(feature = "espsecure")]
                Self::EspSecure => "espsecure.exe",
                #[cfg(feature = "espefuse")]
                Self::EspEfuse => "espefuse.exe",
                #[cfg(feature = "espidfnvs")]
                Self::EspIdfNvs => "espidfnvs.exe",
                _ => unreachable!(),
            }
        } else {
            self.basename()
        }
    }

    /// Return the SHA1 hash of the tool
    const fn sha1(&self) -> &str {
        Self::SHA1[*self as usize]
    }

    /// Return the gzipped binary of the tool
    const fn gz(&self) -> &[u8] {
        Self::GZ_BINARY[*self as usize]
    }
}

impl Display for Tool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.basename())
    }
}

/// A mounted tool
pub struct MountedTool(Tool);

impl MountedTool {
    /// Get the path to the mounted tool
    pub fn path(&self) -> PathBuf {
        MOUNT_POINTS.lock().unwrap().get(&self.0).unwrap().clone()
    }

    /// Execute the mounted tool with the provided arguments.
    ///
    /// # Arguments
    /// * `args` - the arguments to pass to the tool
    ///
    /// # Returns
    /// * `Ok(ExitStatus)` if the tool was executed successfully
    /// * `Err(io::Error)` if the tool could not be executed
    pub fn exec<I>(&self, args: I) -> io::Result<std::process::ExitStatus>
    where
        I: IntoIterator,
        I::Item: AsRef<OsStr>,
    {
        info!("Executing `{}`", self.0);

        let mut cmd = std::process::Command::new(self.path());

        cmd.args(args);

        let result = cmd.status();

        info!("Status {result:?}");

        result
    }
}

struct SliceReader<'a> {
    slice: &'a [u8],
    position: usize,
}

impl<'a> SliceReader<'a> {
    fn new(slice: &'a [u8]) -> Self {
        Self { slice, position: 0 }
    }
}

impl std::io::Read for SliceReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = buf.len().min(self.slice.len() - self.position);

        buf[..n].copy_from_slice(&self.slice[self.position..self.position + n]);

        self.position += n;

        Ok(n)
    }
}

impl std::io::Seek for SliceReader<'_> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let position = match pos {
            std::io::SeekFrom::Start(n) => {
                self.position = n.min(self.slice.len() as u64) as usize;
                return Ok(self.position as u64);
            }
            std::io::SeekFrom::End(n) => self.slice.len() as i64 + n,
            std::io::SeekFrom::Current(n) => self.position as i64 + n,
        };

        if position < 0 {
            self.position = 0;
        } else if position > self.slice.len() as i64 {
            self.position = self.slice.len();
        } else {
            self.position = position as usize;
        }

        Ok(self.position as u64)
    }
}
