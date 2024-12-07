//! A command-line interface to the `esptools` crate.

use std::io::Write as _;

use esptools::{Tool, Tools};

use log::{error, info, LevelFilter};

fn main() {
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter_level(LevelFilter::Info)
        .init();

    let mut args = std::env::args();

    let executable = if let Some(executable) = args.next() {
        if let Some(command) = args.next() {
            let tool = match command.to_ascii_lowercase().as_str() {
                "tool" | "flash" => Tool::EspTool,
                "secure" => Tool::EspSecure,
                "efuse" => Tool::EspEfuse,
                other => {
                    error!("Unknown command `{other}`; must be one of `tool` (or `flash`), `secure`, `efuse`");
                    std::process::exit(1);
                }
            };

            match Tools::mount() {
                Ok(tools) => {
                    if let Err(error) = tools.exec(tool, args) {
                        error!("Failed to execute `{}`: {}", command, error);
                        std::process::exit(1);
                    }
                }
                Err(err) => {
                    error!("Failed to mount tools: {}", err);
                    std::process::exit(1);
                }
            }

            return;
        } else {
            executable
        }
    } else {
        "esptools".to_string()
    };

    info!("Usage: {executable} <command> [<args>]\nWhere <command> is one of `tool`, `secure`, or `efuse`");
}
