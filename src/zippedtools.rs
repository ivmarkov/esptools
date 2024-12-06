use std::fs;
use std::io::{Read, Seek, Write};
use std::path::Path;

use sha1::Digest;

use zip::{read::ZipFile, ZipArchive};

pub type ZipError = zip::result::ZipError;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Tool {
    EspTool = 0,
    EspSecure = 1,
    EspEfuse = 2,
}

impl Tool {
    pub const fn basename(&self) -> &str {
        match self {
            Self::EspTool => "esptool",
            Self::EspSecure => "espsecure",
            Self::EspEfuse => "espefuse",
        }
    }

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

    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::EspTool, Self::EspSecure, Self::EspEfuse].into_iter()
    }
}

pub struct ZippedTools<R> {
    zip: ZipArchive<R>,
    windows: bool,
}

impl<R> ZippedTools<R>
where
    R: Read + Seek,
{
    pub fn new(zip_file: R, windows: bool) -> Result<Self, ZipError> {
        Ok(Self {
            zip: ZipArchive::new(zip_file)?,
            windows,
        })
    }

    pub fn extract(&mut self, tool: Tool, out: &Path) -> Result<(), ZipError> {
        let mut file = fs::File::create(out)?;

        let mut tool_stream = self.find(tool)?;

        let mut buf = [0; 32768];

        loop {
            let n = tool_stream.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }

            file.write_all(&buf[..n]).unwrap();
        }

        Ok(())
    }

    pub fn sha1_of(&mut self, tool: Tool) -> Result<String, ZipError> {
        let mut tool_stream = self.find(tool)?;

        let mut sha1 = sha1::Sha1::new();

        let mut buf = [0; 32768];

        loop {
            let n = tool_stream.read(&mut buf).unwrap();
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
            .unwrap()
            .to_string();

        self.zip.by_name(&name)
    }
}
