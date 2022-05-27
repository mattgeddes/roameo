use clap::{Arg, Command};
use nix::ifaddrs;
use nix::sys::socket::AddressFamily::Inet;
use nix::sys::socket::{SockaddrIn, SockaddrLike};
use std::str::FromStr;
use std::io::{Error,ErrorKind};
use log::debug;


#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(target_os = "linux"))]
mod unsupported;

const ANYINTERFACE: &str = "any";
const EMPTYSTRING: &str = "";
const MAX_INTERFACE_LENGTH: usize = 16;

// configuration data structure
pub struct Roameo {
    interface: String,
    address: String,
    essid: String,
    subnet: String,
}

// Constructor for configuration struct
impl Roameo {
    pub fn new() -> Result<Roameo, &'static str> {
        let args = Command::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::new("essid")
                    .short('e')
                    .long("essid")
                    .takes_value(true)
                    .help("Match wireless ESSID"),
            )
            .arg(
                Arg::new("address")
                    .short('a')
                    .long("address")
                    .takes_value(true)
                    .help("Match IP address (in CIDR form)"),
            )
            .arg(
                Arg::new("subnet")
                    .short('s')
                    .long("subnet")
                    .takes_value(true)
                    .help("Match IP subnet (in CIDR form not implemented)"),
            )
            .arg(
                Arg::new("interface")
                    .short('i')
                    .long("interface")
                    .takes_value(true)
                    .help("Network interface to limit match too. Defaults to all."),
            )
            .get_matches();

        // Return struct
        Ok(Roameo {
            interface: args
                .value_of("interface")
                .unwrap_or(ANYINTERFACE)
                .to_string(),
            address: args.value_of("address").unwrap_or(EMPTYSTRING).to_string(),
            subnet: args.value_of("subnet").unwrap_or(EMPTYSTRING).to_string(),
            essid: args.value_of("essid").unwrap_or(EMPTYSTRING).to_string(),
        })
    }

    pub fn find_match(&self) -> Result<(), Error> {
        if self.interface.len() > MAX_INTERFACE_LENGTH {
            // Bail
            panic!("Interface name longer than maximum allowed");
        }

        if self.essid != EMPTYSTRING {
            return self.match_essid(&self.essid);
        } else if self.subnet != EMPTYSTRING || self.address != EMPTYSTRING {
            return self.get_inet_addrs();
        }
        Err(Error::new(ErrorKind::InvalidInput, "Invalid command line arguments"))
    }

    fn get_inet_addrs(&self) -> Result<(), Error> {
        /*
         * Iterate through (select) interface addresses and return inet4/inet6
         * addresses.
         */
        let addrs = ifaddrs::getifaddrs().unwrap();
        let mut match_addr = self.address.clone();

        if !match_addr.ends_with(":0") {
            match_addr += ":0";
        }

        // Convert string IPv4 address if possible. TODO: make sure that the port
        // is added to the provided address, or from_str() won't parse it.
        let match_ip4 = SockaddrIn::from_str(&match_addr).unwrap_or_else(|_| {
            // match against 0.0.0.0:0 instead
            SockaddrIn::new(0, 0, 0, 0, 0)
        });
        // Try IPv6 to in case that's what we were given.
        /*
            let match_ip6 = SockaddrIn6::from_str(&match_addr).unwrap_or_else(|_| {
                SockaddrIn6::from_str(&"::/0:0").unwrap()
            });
        */

        // Loop through interface addresses
        for ifaddr in addrs {
            // Check to see if we're supposed to limit this to a particular interface
            if ifaddr.interface_name != self.interface && self.interface != ANYINTERFACE {
                continue;
            }

            match ifaddr.address {
                Some(addr) => {
                    debug!(
                        "{}: {:?} {:?}",
                        ifaddr.interface_name,
                        addr.family(),
                        addr.to_string()
                    );
                    if addr.family() == Some(Inet) {
                        let is4 = addr.as_sockaddr_in();
                        if let Some(a4) = is4 {
                            debug!(
                                "{}: IPv4 => {}: Match => {}",
                                ifaddr.interface_name,
                                a4.ip(),
                                match_ip4.ip()
                            );
                            // Try to match only if an address was provided.
                            if self.address != EMPTYSTRING {
                                if a4.ip() == match_ip4.ip() {
                                    debug!(
                                        "Found a match for {} on {}",
                                        self.address, ifaddr.interface_name
                                    );
                                    return Ok(());
                                }
                            }
                        }
                        continue;
                    /*
                                    } else if addr.family() == Some(Inet6) {
                                        let is6 = addr.as_sockaddr_in6();
                                        if let Some(a6) = is6 {
                                            debug!("{}: IPv6 => {}", ifaddr.interface_name, a6.ip());
                                            if self.address != EMPTYSTRING {
                                                if a6.ip() == match_ip6.ip() {
                                                    debug!("Found a match for {} on {}", config.address, ifaddr.interface_name);
                                                }
                                            }
                                        }
                                        continue;
                    */
                    } else {
                        // Skip all others
                        continue;
                    }
                }
                None => {}
            }

            match ifaddr.netmask {
                Some(addr) => {
                    debug!(
                        "{}: {:?} {:?}",
                        ifaddr.interface_name,
                        addr.family(),
                        addr.to_string()
                    );
                }
                None => {}
            }
        }

        return Err(Error::new(
                ErrorKind::Other,
                "Not found"));
    }
}
