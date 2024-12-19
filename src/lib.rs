//! A module for bundling the Espressif `esptools` binaries with a Rust application.
//! See https://github.com/espressif/esptool/releases

use core::fmt::{self, Display};

use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Mutex;
use std::{fs, io};

use log::info;
pub use zippedtools::Tool;
use zippedtools::ZipError;

mod zippedtools;

const WINDOWS: bool = cfg!(target_os = "windows");

static MOUNTED: Mutex<bool> = Mutex::new(false);

impl Tool {
    const fn sha1(&self) -> &str {
        Tools::ESPTOOLS_SHA1[*self as usize]
    }

    fn expanded_name(&self) -> String {
        format!(
            "{}-{}{}",
            self.basename(),
            self.sha1(),
            if WINDOWS { ".exe" } else { "" }
        )
    }
}

/// Error type for the `Tools` struct
#[derive(Debug)]
pub enum ToolsError {
    /// The tools have already been mounted
    AlreadyMounted,
    /// The tools could not be mounted
    MountFailed,
    /// A ZIP error occurred
    Zip(ZipError),
}

impl Display for ToolsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::AlreadyMounted => write!(f, "Tools already created"),
            Self::MountFailed => write!(f, "Tools creation failed"),
            Self::Zip(e) => write!(f, "Zip error: {e}"),
        }
    }
}

impl core::error::Error for ToolsError {}

impl From<ZipError> for ToolsError {
    fn from(e: ZipError) -> Self {
        Self::Zip(e)
    }
}

/// A struct for managing the Espressif `esptools` binaries
pub struct Tools(PathBuf);

impl Tools {
    const ESPTOOLS_ZIP: &[u8] = include_bytes!(env!("ESPTOOLS_ZIP_FILE"));
    //const ESPTOOLS_VERSION: &str = env!("ESPTOOLS_VERSION");

    const ESPTOOLS_SHA1: &[&str] = &[
        env!("ESPTOOL_SHA1"),
        env!("ESPSECURE_SHA1"),
        env!("ESPEFUSE_SHA1"),
    ];

    /// Mount the tools
    ///
    /// This ensures that the tools are expanded in a cache directory private to the crate
    /// and are ready for use.
    ///
    /// Note that the tools can be mounted only once in the application i.e. they are a singleton.
    ///
    /// # Returns
    /// * `Ok(Tools)` if the tools were successfully mounted
    /// * `Err(ToolsError)` if the tools could not be mounted
    pub fn mount() -> Result<Self, ToolsError> {
        let mut mounted = MOUNTED.lock().unwrap();

        if *mounted {
            Err(ToolsError::AlreadyMounted)?;
        }

        let project_dirs = directories::ProjectDirs::from("org", "ivmarkov", "esptools")
            .ok_or(ToolsError::MountFailed)?;

        let tools_dir = project_dirs.cache_dir().to_path_buf();
        let mut zip = None;

        for tool in Tool::iter() {
            let tool_file = tools_dir.join(tool.expanded_name());

            if !tool_file.exists() {
                fs::create_dir_all(tool_file.parent().unwrap()).map_err(ZipError::Io)?;

                if zip.is_none() {
                    zip = Some(zippedtools::ZippedTools::new(
                        SliceReader::new(Self::ESPTOOLS_ZIP),
                        WINDOWS,
                    )?);
                }

                zip.as_mut().unwrap().extract(tool, &tool_file)?;
            }
        }

        *mounted = true;
        info!("Tools mounted in `{}`", tools_dir.display());

        Ok(Self(tools_dir))
    }

    /// Get the path to a tool
    ///
    /// Note that the path is only valid while the `Tools` struct is in scope.
    ///
    /// # Arguments
    /// * `tool` - the tool to get the path for
    pub fn tool_path(&self, tool: Tool) -> PathBuf {
        self.0.join(tool.expanded_name())
    }

    /// Execute a tool with the provided arguments.
    ///
    /// # Arguments
    /// * `tool` - the tool to execute
    /// * `args` - the arguments to pass to the tool
    ///
    /// # Returns
    /// * `Ok(ExitStatus)` if the tool was executed successfully
    /// * `Err(io::Error)` if the tool could not be executed
    pub fn exec<I>(&self, tool: Tool, args: I) -> io::Result<std::process::ExitStatus>
    where
        I: IntoIterator,
        I::Item: AsRef<OsStr>,
    {
        info!("Executing `{tool}`");

        let mut cmd = std::process::Command::new(self.tool_path(tool));

        cmd.args(args);

        let result = cmd.status();

        info!("Status {result:?}");

        result
    }
}

impl Drop for Tools {
    fn drop(&mut self) {
        let mut mounted = MOUNTED.lock().unwrap();
        *mounted = false;

        info!("Tools unmounted");
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
