/*
 * Linux-specific platform code.
 */
use crate::{Roameo, ANYINTERFACE};
use libc::ioctl;
use log::{debug, info};
use network_interface::NetworkInterface;
use network_interface::NetworkInterfaceConfig;
use socket2::{Domain, Socket, Type};
use std::convert::TryInto;
use std::io;
use std::io::{Error, ErrorKind};
use std::os::raw::c_uchar;
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::str;

// The ioctl for getting the ESSID is hardcoded as 0x8b1b in wireless.h
const SIOCGIWESSID: i32 = 0x8b1b;
// This is the maximum length for a network interface name
const IFNAMSIZ: usize = 16;
// This is the maximum length for a Wi-Fi ESSID per wireless.h
const ESSIDSIZ: usize = 32;

// The wireless data structures for ioctls are nested unions. We (currently)
// only care about the ESSID path. Other than that, this is taken almost
// directly from wireless.h in the Linux kernel sources.

// This is the high level structure passed into the ioctl call
#[derive(Debug)]
#[repr(C)]
struct IwReq {
    ifname: [u8; IFNAMSIZ],
    ifdata: IwPoint,
}

// And some helper methods
impl IwReq {
    fn set_ifname(&mut self, interface: &str) {
        let iter = interface.as_bytes().iter();

        if interface.as_bytes().len() > IFNAMSIZ {
            // TODO: better error handling than panic.
            panic!("Interface name too long");
        }

        for (count, b) in iter.enumerate() {
            if count > self.ifname.len() {
                // this case should have been caught by the above check
                panic!("Would have overflowed ifname");
            }
            self.ifname[count] = *b;
        }
    }

    fn set_ifdata(&mut self, b: *const c_uchar) {
        self.ifdata.dlen = ESSIDSIZ as u16;
        self.ifdata.dptr = b;
    }
}

// Default trait to initialise values
impl Default for IwReq {
    fn default() -> IwReq {
        IwReq {
            ifname: [0; IFNAMSIZ],
            ifdata: IwPoint::default(),
        }
    }
}

// This is what the unions evaluate to for the data for the ESSID string
#[derive(Debug)]
#[repr(C)]
struct IwPoint {
    dptr: *const c_uchar,
    dlen: u16,
    flags: u16,
}

impl Default for IwPoint {
    fn default() -> IwPoint {
        IwPoint {
            dptr: ptr::null(),
            dlen: 0_u16,
            flags: 0_u16,
        }
    }
}

impl Roameo {
    pub fn match_essid(&self, essid: &str) -> Result<(), io::Error> {
        let interfaces = NetworkInterface::show().unwrap();

        for i in interfaces.iter() {
            if i.name != self.interface && self.interface != ANYINTERFACE {
                // Skip the interfaces we're not interested in, if one was
                // specified.
                continue;
            }
            debug!("Checking whether {} is connected to {}", i.name, essid);
            match self.has_essid(&i.name, essid) {
                Ok(_) => return Ok(()),
                Err(_) => {
                    continue;
                }
            };
        }

        // We got no match
        Err(Error::new(ErrorKind::Other, "No match"))
    }

    pub fn has_essid(&self, interface: &str, essid: &str) -> Result<(), io::Error> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, None).unwrap();
        let mut req = IwReq::default();
        let buf: [u8; ESSIDSIZ] = [0; ESSIDSIZ];

        req.set_ifname(interface);
        req.set_ifdata(buf.as_ptr() as *const c_uchar);

        unsafe {
            if ioctl(socket.as_raw_fd(), SIOCGIWESSID.try_into().unwrap(), &req) == -1 {
                info!(
                    "ioctl call for {} failed with: {}",
                    interface,
                    io::Error::last_os_error()
                );
                return Err(Error::new(ErrorKind::Other, Error::last_os_error()));
            }
        }

        let mut id = str::from_utf8(&buf).unwrap();
        id = id.trim_matches(char::from(0));
        debug!(
            "Interface {} connected to {:?} ({:?})",
            interface, id, essid
        );
        if id == essid {
            debug!("Matched ESSID {} on interface {}", interface, essid);
            // TODO: Return something here
            return Ok(());
        }
        debug!("ESSID: {}", essid);

        // It's a Wi-Fi interface, but not the one we were looking for.
        Err(Error::new(ErrorKind::NotFound, "Not a match"))
    }
}
