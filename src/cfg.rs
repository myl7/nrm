// Copyright (c) 2022 myl7
// SPDX-License-Identifier: Apache-2.0

use clap::{arg, Command};
use log::LevelFilter;

#[derive(Debug)]
pub struct Cfg {
    pub log_level: LevelFilter,
}

pub fn parse_args(args: &[String]) -> Cfg {
    let matches = Command::new("norm")
        .version("0.1.0")
        .about("Use ptrace to trap unlink* syscall and do path validation to protect your files from sad unexpected unrecoverable deletion")
        .arg(arg!(-Q --quieter "Disable any log"))
        .arg(arg!(-q --quiet "Show error log only"))
        .arg(arg!(-v --verbose "Show debug log"))
        .get_matches_from(args);
    let log_level = if matches.is_present("quieter") {
        LevelFilter::Off
    } else if matches.is_present("quiet") {
        LevelFilter::Error
    } else if matches.is_present("verbose") {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    return Cfg { log_level };
}
