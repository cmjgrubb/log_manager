use std::fs::OpenOptions;
use std::io::prelude::*;
use std::net::UdpSocket;
use std::str;

mod processor;

pub fn syslog() {
    let socket = UdpSocket::bind("0.0.0.0:514")
        .expect("Could not bind to port 514. Ensure that this program is run as sudo.");

    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("/var/log/log_manager");

    let mut file = match file {
        Ok(file) => file,
        Err(e) => {
            eprintln!(
                "Error opening log file: {e}.\nPlease check file permissions, ownership, and path."
            );
            return;
        }
    };

    let mut processor = processor::Processor::new.unwrap();

    loop {
        let mut buf = [0; 1024];
        match socket.recv_from(&mut buf) {
            Ok((amt, _src)) => {
                let text = str::from_utf8(&buf[..amt]).unwrap();
                processor.process_log(text).unwrap();
            }
            Err(e) => {
                writeln!(file, "Error receiving log: {e}").unwrap();
            }
        }
    }
}