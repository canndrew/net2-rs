// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;
use std::io;
use std::mem;
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use libc::c_int;

use sys;
use sys::c;

pub struct Socket {
    inner: sys::Socket,
}

impl Socket {
    pub fn new(family: c_int, ty: c_int) -> io::Result<Socket> {
        Ok(Socket { inner: try!(sys::Socket::new(family, ty)) })
    }

    pub fn bind(&self, addr: &SocketAddr) -> io::Result<()> {
        let (addr, len) = addr2raw(addr);
        unsafe {
            ::cvt(c::bind(self.inner.raw(), addr, len)).map(|_| ())
        }
    }

    pub fn listen(&self, backlog: i32) -> io::Result<()> {
        unsafe {
            ::cvt(c::listen(self.inner.raw(), backlog)).map(|_| ())
        }
    }

    pub fn connect(&self, addr: &SocketAddr) -> io::Result<()> {
        let (addr, len) = addr2raw(addr);
        unsafe {
            ::cvt(c::connect(self.inner.raw(), addr, len)).map(|_| ())
        }
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        let mut addr: c::sockaddr = unsafe { mem::zeroed() };
        let mut addr_len: c::socklen_t = 0;
        try!(unsafe {
            ::cvt(c::getpeername(self.inner.raw(),
                                 &mut addr as *mut _,
                                 &mut addr_len as *mut _)).map(|_| ())
        });
        Ok(raw2addr(&addr as *const _, addr_len))
    }
}

impl fmt::Debug for Socket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.raw().fmt(f)
    }
}

impl ::AsInner for Socket {
    type Inner = sys::Socket;
    fn as_inner(&self) -> &sys::Socket { &self.inner }
}

impl ::FromInner for Socket {
    type Inner = sys::Socket;
    fn from_inner(sock: sys::Socket) -> Socket {
        Socket { inner: sock }
    }
}

impl ::IntoInner for Socket {
    type Inner = sys::Socket;
    fn into_inner(self) -> sys::Socket { self.inner }
}

fn addr2raw(addr: &SocketAddr) -> (*const c::sockaddr, c::socklen_t) {
    match *addr {
        SocketAddr::V4(ref a) => {
            (a as *const _ as *const _, mem::size_of_val(a) as c::socklen_t)
        }
        SocketAddr::V6(ref a) => {
            (a as *const _ as *const _, mem::size_of_val(a) as c::socklen_t)
        }
    }
}

#[allow(unused)]
fn raw2addr(addr: *const c::sockaddr, len: c::socklen_t) -> SocketAddr {
    if len as usize == mem::size_of::<SocketAddrV4>() {
        unsafe {
            let addr: *const SocketAddrV4 = mem::transmute(addr);
            SocketAddr::V4(*addr)
        }
    }
    else {
        unsafe {
            let addr: *const SocketAddrV6 = mem::transmute(addr);
            SocketAddr::V6(*addr)
        }
    }
}

#[cfg(test)]
#[test]
fn test_raw_and_back() {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)), 56);
    let (raw, raw_len) = addr2raw(&addr);
    let back = raw2addr(raw, raw_len);
    assert_eq!(addr, back);

    let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8)), 56);
    let (raw, raw_len) = addr2raw(&addr);
    let back = raw2addr(raw, raw_len);
    assert_eq!(addr, back);
}

