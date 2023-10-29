/*
 * Platform-specific code for platforms that don't yet support certain
 * functionality.
 */

use crate::Roameo;
use std::io::{Error, ErrorKind};

impl Roameo {
    pub fn match_essid(&self, _essid: &str) -> Result<(), Error> {
        Err(Error::new(
            ErrorKind::Other,
            "ESSID match support not yet available on this platform",
        ))
    }
}
