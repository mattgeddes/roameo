use clap::{Arg, Command};
use log::debug;
use nix::ifaddrs;
use nix::sys::socket::AddressFamily::Inet;
use nix::sys::socket::{SockaddrIn, SockaddrLike, SockaddrStorage};
use std::fmt;
use std::io::{Error, ErrorKind};
use std::str::FromStr;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(target_os = "linux"))]
mod unsupported;

const ANYINTERFACE: &str = "any";
const EMPTYSTRING: &str = "";
const MAX_INTERFACE_LENGTH: usize = 16;

/// IPv4NetworkAddress is a struct to hide the munging of addresses
#[derive(Eq)]
pub struct IPv4NetworkAddress {
    addr: u32,
    mask: u32,
    network: u32,
}

impl PartialEq for IPv4NetworkAddress {
    fn eq(&self, other: &Self) -> bool {
        self.network == other.network
    }
}

impl fmt::Display for IPv4NetworkAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:08x}/{:08x}", self.addr, self.mask)
    }
}

impl IPv4NetworkAddress {
    pub fn from_sockaddr(
        addr: &SockaddrStorage,
        mask: &SockaddrStorage,
    ) -> Result<IPv4NetworkAddress, Error> {
        let mut ret_addr = 0;
        let mut ret_mask = 0;
        if let Some(in_addr) = addr.as_sockaddr_in() {
            ret_addr = in_addr.ip();
        }
        if let Some(in_mask) = mask.as_sockaddr_in() {
            ret_mask = in_mask.ip();
        }
        debug!("Parsed address to {}", ret_addr);
        debug!("Parsed netmask to {}", ret_mask);
        Ok(IPv4NetworkAddress {
            addr: ret_addr,
            mask: ret_mask,
            network: ret_addr & ret_mask,
        })
    }
    pub fn from_cidr(cidr: &str) -> Result<IPv4NetworkAddress, Error> {
        let mut ret_addr = 0;
        let mut ret_mask = 0;

        let t = cidr.split_once('/');
        if let Some((addr, mask)) = t {
            let octets = addr.splitn(4, '.');
            for (count, octet) in octets.enumerate() {
                let bits = 8 * (3 - count);
                let num: u32 = octet.parse().unwrap();
                ret_addr += num << bits;
            }

            debug!("Parsed address {} to {}", addr, ret_addr);

            // This isn't the most efficient, but will work.
            let mask_bits: u32 = mask.parse().unwrap();
            // Do 1s first
            for i in 0..32 {
                if i >= (32 - mask_bits) {
                    ret_mask |= 1 << i;
                }
            }

            debug!("Parsed netmask {} to {}", mask, ret_mask);
        }
        // split cidr once at '/'
        // split addr by '.' and for each, shift left (ndots * 8) and add
        Ok(IPv4NetworkAddress {
            addr: ret_addr,
            mask: ret_mask,
            network: ret_addr & ret_mask,
        })
    }
}

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
                    .help("Match wireless ESSID. Eg, 'CorporateWiFi`"),
            )
            .arg(
                Arg::new("address")
                    .short('a')
                    .long("address")
                    .takes_value(true)
                    .help("Match IP address. Eg, '203.0.113.6'."),
            )
            .arg(
                Arg::new("subnet")
                    .short('s')
                    .long("subnet")
                    .takes_value(true)
                    .help("Match IP subnet. Eg, '203.0.113.0/24'."),
            )
            .arg(
                Arg::new("interface")
                    .short('i')
                    .long("interface")
                    .takes_value(true)
                    .help("Network interface to limit match too. Eg, 'eno1'. Defaults to all."),
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
        Err(Error::new(
            ErrorKind::InvalidInput,
            "Invalid command line arguments",
        ))
    }

    fn get_inet_addrs(&self) -> Result<(), Error> {
        /*
         * Iterate through (select) interface addresses and return inet4/inet6
         * addresses.
         */
        let addrs = ifaddrs::getifaddrs()?;
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

            if let Some(addr) = ifaddr.address {
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
                        if self.address != EMPTYSTRING && a4.ip() == match_ip4.ip() {
                            debug!(
                                "Found a match for {} on {}",
                                self.address, ifaddr.interface_name
                            );
                            return Ok(());
                        } else if self.subnet != EMPTYSTRING {
                            debug!(
                                "Trying to find a match for subnet {} on {}",
                                self.subnet, ifaddr.interface_name,
                            );
                            if let Some(s) = ifaddr.netmask {
                                let dest = IPv4NetworkAddress::from_sockaddr(&addr, &s)?;
                                let src = IPv4NetworkAddress::from_cidr(&self.subnet)?;
                                if src == dest {
                                    debug!("Found subnet: {}", dest);
                                    return Ok(());
                                }
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
                    // Skip all other address types
                    continue;
                }
            }

            if let Some(addr) = ifaddr.netmask {
                debug!(
                    "{}: {:?} {:?}",
                    ifaddr.interface_name,
                    addr.family(),
                    addr.to_string()
                );
            }
        }

        Err(Error::new(ErrorKind::Other, "Not found"))
    }
}
