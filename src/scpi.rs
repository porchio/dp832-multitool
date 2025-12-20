// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2024 Marcus Hoffmann

/// SCPI Communication Module
/// 
/// Provides low-level SCPI communication primitives for the DP832 power supply.

use std::io::{Read, Write};
use std::net::TcpStream;

/// Send a SCPI command to the device
pub fn send(stream: &mut TcpStream, cmd: &str) {
    let cmd = format!("{}\n", cmd);
    stream.write_all(cmd.as_bytes()).unwrap();
}

/// Send a SCPI query and read the response
pub fn query(stream: &mut TcpStream, cmd: &str) -> String {
    send(stream, cmd);
    let mut resp = Vec::new();
    let mut buf = [0u8; 64];

    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                resp.extend_from_slice(&buf[..n]);
                if resp.ends_with(b"\n") {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => panic!("{}", e),
        }
    }

    String::from_utf8_lossy(&resp).trim().to_string()
}
