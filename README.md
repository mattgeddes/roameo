# Roameo

## Overview

This small Rust project is a wrapper around a few ioctls and similar to make it
easier to test for certain platform state quickly and easily. Examples include
being able to test whether we're currently connected to a specific Wi-Fi SSID.

It is intended for use in cases such as `ssh_config(5)`'s `Match exec` clause,
which would allow different OpenSSH client configurations, depending on which
wireless network we're currently connected to. For example, using the
`ProxyJump` configuration option to go through a jump host when on a network
other than the corporate/office network.

This could be done with a few lines of shell script wrapped around command line tools, but I was looking for an excuse to write some Rust code, and going direct with ioctls is less likely to break.

## Supported Platforms

Linux is my primary operating system and is the best supported. I do also run and test this code on MacOS, OpenBSD and FreeBSD too, but some of the functionality (ESSID matching for example) are lagging behind a little.

The intent is to support anything Unix-like pretty-well equally.

## Example Configuration

Here's an example `ssh_config(5)` configuration fragment to illustrate how this code might be used:

```
Match host 10.0.0.? !exec "roameo -e CorporateWiFi"
    ProxyJump me@jumphost.corp.net:2222
    ForwardAgent yes
    DynamicForward 3128
```

This hypothetical example:
1. Matches hosts on the 10.0.0.0/24 subnet -- presumably our hypothetical corporate network subnet, and
2. Uses `Match exec` with roameo to match the case where we are *not* on the Wi-Fi network called CorporateWiFi -- presumably our hypothetical corporate network Wi-Fi network ESSID.

Essentially, this gives us specific SSH client configuration for the case where we're trying to access corporate resources, but from a network other than the corporate network.

The example then goes on to set a jump host, agent forwarding and SOCKS5 proxy tunnelling automatically. Whereas when we're on the corporate network, these would not necessarily apply.


# Future Functionality

The initial version only supports matching against an ESSID or a specific source IP address. Functionality planned but not yet implemented includes:

1. Matching against source subnet
2. Matching against IPv6 addresses and subnets
3. Matching any Wi-Fi connectivity, or any IPv6 (global) addressing
4. Matching VPN and other tunnels
5. Better support for non-Linux platforms

Comments and pull requests welcome.

