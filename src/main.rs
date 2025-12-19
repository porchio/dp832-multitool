use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;

const DP832_ADDR: &str = "192.168.1.140:5555"; // <-- change this
const CHANNEL: u8 = 1;

const VOC: f64 = 4.20;      // Open-circuit voltage (V)
const R_INT: f64 = 0.120;   // Internal resistance (ohms)
const I_MAX: f64 = 0.2;     // Max current (A)
const UPDATE_MS: u64 = 200; // Update interval

fn send(stream: &mut TcpStream, cmd: &str) {
    let full = format!("{}\n", cmd);
    stream.write_all(full.as_bytes()).unwrap();
}

fn query(stream: &mut TcpStream, cmd: &str) -> String {
    send(stream, cmd);

    let mut response = Vec::new();
    let mut buf = [0u8; 64];

    loop {
        match stream.read(&mut buf) {
            Ok(0) => break, // connection closed
            Ok(n) => {
                response.extend_from_slice(&buf[..n]);
                if response.ends_with(b"\n") {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                break;
            }
            Err(e) => panic!("Read error: {}", e),
        }
    }

    String::from_utf8_lossy(&response).trim().to_string()
}

fn main() {
    println!("Connecting to DP832...");
    let mut stream = TcpStream::connect(DP832_ADDR).unwrap();
    stream.set_read_timeout(Some(Duration::from_millis(500))).unwrap();

    // Identify
    let idn = query(&mut stream, "*IDN?");
    println!("Connected to: {}", idn);

    // Basic setup
    send(&mut stream, &format!("INST:NSEL {}", CHANNEL));
    send(&mut stream, "OUTP OFF");
    send(&mut stream, &format!("CURR {}", I_MAX));
    send(&mut stream, &format!("VOLT {}", VOC));
    send(&mut stream, "OUTP ON");

    println!("Battery simulation started");

    loop {
        // Measure current
        let i_str = query(&mut stream, "MEAS:CURR?");
        let i_load: f64 = i_str.parse().unwrap_or(0.0);

        // Compute battery voltage
        let mut v_out = VOC - i_load * R_INT;

        if v_out < 0.0 {
            v_out = 0.0;
        }

        // Apply voltage
        send(&mut stream, &format!("VOLT {:.3}", v_out));

        println!(
            "I_load = {:.3} A | V_out = {:.3} V",
            i_load, v_out
        );

        sleep(Duration::from_millis(UPDATE_MS));
    }
}

