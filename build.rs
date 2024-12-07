use std::fs;
use std::path::PathBuf;

use zippedtools::{Tool, ZippedTools};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

const GIT_REPO: &str = "https://github.com/espressif/esptool/releases/download";

// The version of the tools to download
// Updating this version means new `esptools` patch release
const VERSION: &str = "v4.8.1";

const PREFIX: &str = "esptool";
const SUFFIX_WIN64: &str = "win64";
const SUFFIX_MACOS: &str = "macos";
const SUFFIX_LINUX_AMD64: &str = "linux-amd64";
const SUFFIX_LINUX_ARM64: &str = "linux-arm64";
const SUFFIX_LINUX_ARM32: &str = "linux-arm32";

#[path = "src/zippedtools.rs"]
mod zippedtools;

/// Download the `esptools` ZIP file and bundle it with this crate using `include_bytes!`
fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let image = match target_os.as_str() {
        "windows" => (target_arch == "x86_64").then_some(SUFFIX_WIN64),
        "macos" => (target_arch == "x86_64").then_some(SUFFIX_MACOS),
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

    let client = reqwest::blocking::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .unwrap();

    let zip_response = client
        .get(format!(
            "{GIT_REPO}/{VERSION}/{PREFIX}-{VERSION}-{image}.zip"
        ))
        .send()
        .unwrap();

    let zip_file_path = out.join("esptools.zip");

    fs::write(&zip_file_path, zip_response.bytes().unwrap()).unwrap();

    let zip_file_path = zip_file_path.canonicalize().unwrap();

    println!("cargo:rerun-if-changed={}", zip_file_path.display());
    println!(
        "cargo:rustc-env=ESPTOOLS_ZIP_FILE={}",
        zip_file_path.display()
    );
    println!("cargo:rustc-env=ESPTOOLS_VERSION={VERSION}");

    let mut tools = ZippedTools::new(
        fs::File::open(&zip_file_path).unwrap(),
        target_os.as_str() == "windows",
    )
    .unwrap();

    for tool in Tool::iter() {
        let sha1 = tools.sha1_of(tool).unwrap();
        println!(
            "cargo:rustc-env={}_SHA1={sha1}",
            tool.basename().to_ascii_uppercase()
        );
    }
}
