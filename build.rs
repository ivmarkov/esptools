use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

use sha1::Digest;

use tar::Archive;

use zip::ZipArchive;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

const GIT_REPO_ESPTOOL: &str = "https://github.com/espressif/esptool/releases/download";
const PREFIX_ESPTOOL: &str = "esptool";
// The version of the tools to download
// Updating this version means new `esptools` patch release
const VERSION_ESPTOOL: &str = "v4.8.1";

const GIT_REPO_ESPIDFNVS: &str =
    "https://github.com/ivmarkov/esp-idf-nvs-partition-gen/releases/download";
const PREFIX_ESPIDFNVS: &str = "espidfnvs";
// Ditto for the `espidfnvs` tool which is a separate ZIP file
const VERSION_ESPIDFNVS: &str = "v0.0.1";

const SUFFIX_WIN64: &str = "win64";
const SUFFIX_MACOS: &str = "macos";
const SUFFIX_LINUX_AMD64: &str = "linux-amd64";
const SUFFIX_LINUX_ARM64: &str = "linux-arm64";
const SUFFIX_LINUX_ARM32: &str = "linux-arm32";

const SUFFIX_ESPIDFNVS_WIN64: &str = SUFFIX_WIN64;
const SUFFIX_ESPIDFNVS_LINUX_AMD64: &str = SUFFIX_LINUX_AMD64;
const SUFFIX_ESPIDFNVS_LINUX_ARM64: &str = "aarch64"; // TODO: Fix this in `esp-idf-nvs-partition-gen`
const SUFFIX_ESPIDFNVS_LINUX_ARM32: &str = "armv7"; // TODO: Fix this in `esp-idf-nvs-partition-gen`
const SUFFIX_ESPIDFNVS_MACOS_AMD64: &str = "macos-amd64";
const SUFFIX_ESPIDFNVS_MACOS_ARM64: &str = "macos-arm64";

/// Download the `esptools` & `espidfnvs` archive files and bundle
/// the executables inside those with this crate using `include_bytes!`
fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();
    let windows = target_os.as_str() == "windows";

    let esptool = std::env::var("CARGO_FEATURE_ESPTOOL").is_ok();
    let espsecure = std::env::var("CARGO_FEATURE_ESPSECURE").is_ok();
    let espefuse = std::env::var("CARGO_FEATURE_ESPEFUSE").is_ok();

    let espidfnvs = std::env::var("CARGO_FEATURE_ESPIDFNVS").is_ok();

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    if esptool || espsecure || espefuse {
        let image = match target_os.as_str() {
            "windows" => (target_arch == "x86_64").then_some(SUFFIX_WIN64),
            "macos" => {
                (target_arch == "x86_64" || target_arch == "aarch64").then_some(SUFFIX_MACOS)
            }
            "linux" => (target_env == "gnu")
                .then_some(match target_arch.as_str() {
                    "x86_64" => Some(SUFFIX_LINUX_AMD64),
                    "aarch64" => Some(SUFFIX_LINUX_ARM64),
                    "arm" => Some(SUFFIX_LINUX_ARM32),
                    _ => None,
                })
                .flatten(),
            _ => None,
        };

        let Some(image) = image else {
            panic!("Unsupported target: os={target_os}, arch={target_arch}, env={target_env}");
        };

        process_archive(
            &out,
            &format!(
                "{GIT_REPO_ESPTOOL}/{VERSION_ESPTOOL}/{PREFIX_ESPTOOL}-{VERSION_ESPTOOL}-{image}.zip"
            ),
            &format!("{PREFIX_ESPTOOL}-{image}"),
            windows,
            esptool
                .then_some("esptool")
                .into_iter()
                .chain(espsecure.then_some("espsecure"))
                .chain(espefuse.then_some("espefuse"))
                .collect::<Vec<_>>()
                .as_slice(),
        );
    }

    if espidfnvs {
        let image = match target_os.as_str() {
            "windows" => (target_arch == "x86_64").then_some(SUFFIX_ESPIDFNVS_WIN64),
            "macos" if target_arch == "x86_64" => Some(SUFFIX_ESPIDFNVS_MACOS_AMD64),
            "macos" if target_arch == "aarch64" => Some(SUFFIX_ESPIDFNVS_MACOS_ARM64),
            "linux" => (target_env == "gnu")
                .then_some(match target_arch.as_str() {
                    "x86_64" => Some(SUFFIX_ESPIDFNVS_LINUX_AMD64),
                    "aarch64" => Some(SUFFIX_ESPIDFNVS_LINUX_ARM64),
                    "arm" => Some(SUFFIX_ESPIDFNVS_LINUX_ARM32),
                    _ => None,
                })
                .flatten(),
            _ => None,
        };

        let Some(image) = image else {
            panic!("Unsupported target: os={target_os}, arch={target_arch}, env={target_env}");
        };

        process_archive(
            &out,
            &format!("{GIT_REPO_ESPIDFNVS}/{VERSION_ESPIDFNVS}/{PREFIX_ESPIDFNVS}-{VERSION_ESPIDFNVS}-{image}{}", if windows { ".zip" } else { ".tar.gz" }),
            &format!("{PREFIX_ESPIDFNVS}-{image}"),
            windows,
            &["espidfnvs"],
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn process_archive(
        out: &Path,
        archive_url: &str,
        archive_name: &str,
        windows: bool,
        appl_tools: &[&str],
    ) {
        fn encode<R, W>(mut read: R, write: W) -> String
        where
            R: Read,
            W: Write,
        {
            let mut write = GzEncoder::new(write, flate2::Compression::default());

            let mut sha1 = sha1::Sha1::new();

            let mut buf = [0; 32768];

            loop {
                let n = read.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }

                write.write_all(&buf[..n]).unwrap();

                sha1.update(&buf[..n]);
            }

            hex::encode(sha1.finalize().as_slice())
        }

        let tool_name = |tool: &str| {
            if windows {
                format!("{tool}.exe")
            } else {
                tool.to_string()
            }
        };

        let client = reqwest::blocking::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()
            .unwrap();

        let mut response = client.get(archive_url).send().unwrap();

        let archive_process_path = out.join(archive_name);
        fs::create_dir_all(&archive_process_path).unwrap();

        let archive_path = archive_process_path.join("archive");
        io::copy(&mut response, &mut File::create(&archive_path).unwrap()).unwrap();

        let tools_path = archive_process_path.join("tools");
        fs::create_dir_all(&tools_path).unwrap();

        if archive_url.ends_with(".zip") {
            let mut zip = ZipArchive::new(File::open(&archive_path).unwrap()).unwrap();

            for tool in appl_tools {
                let mut entry = zip
                    .by_name(&format!("{archive_name}/{}", tool_name(tool)))
                    .unwrap();
                let tool_path = tools_path.join(tool_name(tool));
                let mut file = fs::File::create(&tool_path).unwrap();

                let sha1 = encode(&mut entry, &mut file);

                println!(
                    "cargo:rustc-env=TOOL_{}={}",
                    tool.to_ascii_uppercase(),
                    tool_path.display()
                );
                println!(
                    "cargo:rustc-env=TOOL_{}_SHA1={sha1}",
                    tool.to_ascii_uppercase()
                );
            }
        } else if archive_url.ends_with(".tar.gz") {
            let mut archive = Archive::new(GzDecoder::new(File::open(&archive_path).unwrap()));

            for entry in archive.entries().unwrap() {
                let mut entry = entry.unwrap();
                let entry_path = entry.path().unwrap().to_path_buf();

                if let Some(filename) = entry_path
                    .file_name()
                    .and_then(|filename| filename.to_str())
                {
                    if let Some(tool) = appl_tools.iter().find(|tool| tool_name(tool) == filename) {
                        let tool_path = tools_path.join(filename);
                        let mut file = fs::File::create(&tool_path).unwrap();

                        let sha1 = encode(&mut entry, &mut file);

                        println!(
                            "cargo:rustc-env=TOOL_{}={}",
                            tool.to_ascii_uppercase(),
                            tool_path.display()
                        );
                        println!(
                            "cargo:rustc-env=TOOL_{}_SHA1={sha1}",
                            tool.to_ascii_uppercase()
                        );
                    }
                }
            }
        } else {
            panic!("Unsupported archive format: {}", archive_name);
        }
    }
}
