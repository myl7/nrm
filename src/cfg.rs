// Copyright (c) 2022 myl7
// SPDX-License-Identifier: Apache-2.0

use clap::{arg, Command};
use log::LevelFilter;

#[derive(Debug)]
pub struct Cfg {
    pub log_level: LevelFilter,
    pub cmd: Vec<String>,
}

pub fn parse_args() -> Cfg {
    let matches = Command::new("nrm")
        .version("0.1.0")
        .about("Use ptrace to trap unlink* syscall and do path validation to protect your files from sad unexpected unrecoverable deletion")
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .arg(arg!(-Q --quieter "Disable any log"))
        .arg(arg!(-q --quiet "Show error log only"))
        .arg(arg!(-v --verbose "Show debug log"))
        .get_matches();
    let log_level = if matches.is_present("quieter") {
        LevelFilter::Off
    } else if matches.is_present("quiet") {
        LevelFilter::Error
    } else if matches.is_present("verbose") {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    let cmd = match matches.subcommand() {
        Some((cmd, sub_matches)) => {
            let mut args = sub_matches
                .values_of("")
                .unwrap_or_default()
                .map(|s| s.to_owned())
                .collect::<Vec<_>>();
            args.insert(0, cmd.to_owned());
            args
        }
        _ => Vec::new(),
    };
    return Cfg { log_level, cmd };
}
