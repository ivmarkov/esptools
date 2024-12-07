use core::fmt::{self, Display};

use std::fs;
use std::io::{Read, Seek, Write};
use std::path::Path;

use log::{debug, info};

use sha1::Digest;

use zip::{read::ZipFile, ZipArchive};

pub type ZipError = zip::result::ZipError;

/// ESP tools supported by this crate
#[allow(clippy::enum_variant_names)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Tool {
    /// `esptool.py`
    EspTool = 0,
    /// `espsecure.py`
    EspSecure = 1,
    /// `espefuse.py`
    EspEfuse = 2,
}

impl Tool {
    /// Return the base file name of the tool
    pub const fn basename(&self) -> &str {
        match self {
            Self::EspTool => "esptool",
            Self::EspSecure => "espsecure",
            Self::EspEfuse => "espefuse",
        }
    }

    /// Return the file name of the tool
    pub const fn name(&self, windows: bool) -> &str {
        if windows {
            match self {
                Self::EspTool => "esptool.exe",
                Self::EspSecure => "espsecure.exe",
                Self::EspEfuse => "espefuse.exe",
            }
        } else {
            self.basename()
        }
    }

    /// Return an iterator over all tools
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::EspTool, Self::EspSecure, Self::EspEfuse].into_iter()
    }
}

impl Display for Tool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.basename())
    }
}

/// Utility for working with the tools' ZIP file, as downloaded from
/// https://github.com/espressif/esptool/releases
pub(crate) struct ZippedTools<R> {
    zip: ZipArchive<R>,
    windows: bool,
}

#[allow(unused)]
impl<R> ZippedTools<R>
where
    R: Read + Seek,
{
    /// Create a new `ZippedTools` instance from a ZIP file
    pub fn new(zip_file: R, windows: bool) -> Result<Self, ZipError> {
        Ok(Self {
            zip: ZipArchive::new(zip_file)?,
            windows,
        })
    }

    /// Extract a tool from the ZIP file
    ///
    /// Arguments:
    /// * `tool` - the tool to extract
    /// * `out` - the path to extract the tool to
    pub fn extract(&mut self, tool: Tool, out: &Path) -> Result<(), ZipError> {
        debug!("Extracting {tool} to {}", out.display());

        let mut file = fs::File::create(out)?;

        let mut tool_stream = self.find(tool)?;

        let mut buf = [0; 32768];

        loop {
            let n = tool_stream.read(&mut buf)?;
            if n == 0 {
                break;
            }

            file.write_all(&buf[..n])?;
        }

        #[cfg(unix)]
        {
            use std::fs::Permissions;
            use std::os::unix::fs::PermissionsExt;

            std::fs::set_permissions(out, Permissions::from_mode(0o755)).map_err(ZipError::Io)?;
        }

        Ok(())
    }

    /// Compute the SHA-1 signature of a tool
    ///
    /// Arguments:
    /// * `tool` - the tool to compute the signature of
    pub fn sha1_of(&mut self, tool: Tool) -> Result<String, ZipError> {
        info!("Computing signature of {tool}");

        let mut tool_stream = self.find(tool)?;

        let mut sha1 = sha1::Sha1::new();

        let mut buf = [0; 32768];

        loop {
            let n = tool_stream.read(&mut buf)?;
            if n == 0 {
                break;
            }

            sha1.update(&buf[..n]);
        }

        Ok(hex::encode(sha1.finalize().as_slice()))
    }

    fn find(&mut self, tool: Tool) -> Result<ZipFile, ZipError> {
        let name = self
            .zip
            .file_names()
            .find(|name| name.ends_with(tool.name(self.windows)))
            .ok_or(ZipError::FileNotFound)?
            .to_string();

        self.zip.by_name(&name)
    }
}
