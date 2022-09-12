use crate::errors::exit_with_retcode;

use std::process::exit;
use crate::container::Container;

#[macro_use] extern crate scan_fmt;

mod capabilities;
mod child;
mod cli;
mod config;
mod container;
mod errors;
mod hostname;
mod ipc;
mod mounts;
mod namespaces;
mod resources;
mod syscalls;

fn main() {
    match cli::parse_args() {
        Ok(args) => {
            log::info!("{:?}", args);
            exit_with_retcode(Container::start(args));
        },
        Err(e) => {
            log::error!("Error while parsing arguments:\n\t{}", e);
            exit(e.get_retcode());
        }
    }
}
