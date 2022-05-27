#![warn(missing_docs)]
//! # roameo
//!
//! #![doc = include_str!("README.md")]
//!
//! ## License
//!
//! GPL-3.0-only See [LICENSE][LICENSE] for specifics.
use roameo::Roameo;
use std::process;
use log::{error,info};

fn main() {
    env_logger::init();
    // Parse command line arguments
    let r = Roameo::new().unwrap_or_else(|err| {
        error!("Unable to parse arguments: {}", err);
        process::exit(exitcode::USAGE);
    });

    // Do the thing
    if let Err(e) = r.find_match() {
        error!("Error: {}", e);
        process::exit(exitcode::DATAERR);
    }

    info!("Matched. Returning success.");
    process::exit(exitcode::OK);
}