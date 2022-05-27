/*
 * Platform-specific code for platforms that don't yet support certain
 * functionality.
 */

use std::io::{Error, ErrorKind};
use crate::Roameo;

impl Roameo {
    pub fn match_essid(&self) -> Result<(), Error> {
        Err(Error::new(
            ErrorKind::Other,
            "ESSID match support not yet available on this platform",
        ))
    }
}
