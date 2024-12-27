//! A command-line interface to the `esptools` crate.

use std::io::Write as _;

use esptools::Tool;

use log::{error, info, LevelFilter};

fn main() {
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter_level(LevelFilter::Info)
        .init();

    let mut args = std::env::args();

    let executable = if let Some(executable) = args.next() {
        if let Some(command) = args.next() {
            let Some(tool) = Tool::iter().find(|tool| tool.cmd_matches(&command)) else {
                error!(
                    "Unknown command `{command}`; must be one of {}",
                    Tool::iter()
                        .map(|tool| tool.cmd_description().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                std::process::exit(1);
            };

            match tool.mount() {
                Ok(tool) => {
                    if let Err(error) = tool.exec(args) {
                        error!("Failed to execute `{}`: {}", command, error);
                        std::process::exit(1);
                    }
                }
                Err(err) => {
                    error!("Failed to mount tool: {}", err);
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

    info!(
        "Usage: {executable} <command> [<args>]\nWhere <command> is one of {}",
        Tool::iter()
            .map(|tool| tool.cmd_description().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
}
