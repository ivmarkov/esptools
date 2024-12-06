use std::{path::PathBuf, sync::Mutex};

pub use zippedtools::Tool;

mod zippedtools;

const WINDOWS: bool = cfg!(target_os = "windows");

static MOUNTED: Mutex<bool> = Mutex::new(false);

impl Tool {
    const fn sha1(&self) -> &str {
        Tools::ESPTOOLS_SHA1[*self as usize]
    }

    fn expanded_name(&self, windows: bool) -> String {
        format!(
            "{}-{}{}",
            self.basename(),
            self.sha1(),
            if windows { ".exe" } else { "" }
        )
    }
}

#[derive(Debug)]
pub enum ToolsError {
    AlreadyCreated,
}

pub struct Tools(PathBuf);

impl Tools {
    const ESPTOOLS_ZIP: &[u8] = include_bytes!(env!("ESPTOOLS_ZIP_FILE"));
    //const ESPTOOLS_VERSION: &str = env!("ESPTOOLS_VERSION");

    const ESPTOOLS_SHA1: &[&str] = &[
        env!("ESPTOOL_SHA1"),
        env!("ESPSECURE_SHA1"),
        env!("ESPEFUSE_SHA1"),
    ];

    pub fn mount() -> Result<Self, ToolsError> {
        let mut mounted = MOUNTED.lock().unwrap();

        if *mounted {
            Err(ToolsError::AlreadyCreated)?;
        }

        let project_dirs = directories::ProjectDirs::from("org", "ivmarkov", "esptools").unwrap();

        let tools_dir = project_dirs.cache_dir().to_path_buf();
        let mut zip = None;

        for tool in Tool::iter() {
            let tool_file = tools_dir.join(tool.expanded_name(WINDOWS));

            if !tool_file.exists() {
                if zip.is_none() {
                    zip = Some(
                        zippedtools::ZippedTools::new(
                            SliceReader::new(Self::ESPTOOLS_ZIP),
                            WINDOWS,
                        )
                        .unwrap(),
                    );
                }

                zip.as_mut().unwrap().extract(tool, &tool_file).unwrap();

                // TODO: chown
            }
        }

        *mounted = true;

        Ok(Self(tools_dir))
    }

    pub fn exec(&self, tool: Tool, args: &[&str]) -> std::process::ExitStatus {
        let tool_file = self.0.join(tool.expanded_name(WINDOWS));

        let mut cmd = std::process::Command::new(tool_file);

        cmd.args(args);

        cmd.status().unwrap()
    }
}

impl Drop for Tools {
    fn drop(&mut self) {
        let mut mounted = MOUNTED.lock().unwrap();
        *mounted = false;
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

impl<'a> std::io::Read for SliceReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = buf.len().min(self.slice.len() - self.position);

        buf[..n].copy_from_slice(&self.slice[self.position..self.position + n]);

        self.position += n;

        Ok(n)
    }
}

impl<'a> std::io::Seek for SliceReader<'a> {
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
