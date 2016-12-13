#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate hyper;

use regex::Regex;

use std::io::prelude::*;

use std::net::{TcpListener};

use hyper::client::Client;
use hyper::status::StatusCode;

#[derive(Debug, Copy, Clone)]
pub struct LogEntry<'a> {
    value: &'a str,
}

impl<'a> LogEntry<'a> {
    fn new(v: &str) -> LogEntry {
        LogEntry{value: v}
    }

    fn parse_mac_address(self) -> Result<&'a str, &'static str> {
        lazy_static! {
            static ref MAC: Regex = Regex::new(r"([0-9a-f]{2}:[0-9a-f]{2}:[0-9a-f]{2}:[0-9a-f]{2}:[0-9a-f]{2}:[0-9a-f]{2})").unwrap();
        }

        let mut cap = MAC.captures_iter(&self.value);
        let first = match cap.nth(0) {
            Some(c) => c,
            None => return Err("No mac address present"),
        };

        first.at(0).ok_or("No mac address present")
    }

    fn forward(self, mac_address: &'a str, host: &'a str) -> Result<StatusCode, StatusCode> {
        let client = Client::new();
        let url = format!("https://{}/{}", host, mac_address);

        match client.post(&url[..]).send() {
            Ok(r) => Ok(r.status),
            Err(_) => Err(StatusCode::ServiceUnavailable),
        }
    }
}

fn main() {
    const PORT: i32 = 10514;

    let socket = format!("127.0.0.1:{}", PORT);
    let listener = TcpListener::bind(&socket[..]).expect(&format!("Cannot establish connection on port {}", PORT));

    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                let mut stream = String::new();

                s.read_to_string(&mut stream).unwrap();

                let le = LogEntry::new(&stream[..]);

                let mac = match le.parse_mac_address() {
                    Ok(m) => m,
                    Err(e) => {
                        println!("Failed: {}", e);
                        continue;
                    },
                };

                match le.forward(&mac, "wrestlers.hhd.com.au") {
                    Ok(_) => println!("Sent: {}", mac),
                    Err(_) => println!("Failed: {}", mac),
                }
            },
            Err(_) => continue,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::status::StatusCode;

    #[test]
    fn new_log_entry() {
        let le = LogEntry::new("test");

        assert_eq!(le.value, "test");
    }

    #[test]
    fn parse_mac_address_die() {
        let le = LogEntry::new("this is invalid");

        assert!(le.parse_mac_address().is_err());
        assert_eq!(le.parse_mac_address(), Err("No mac address present"));
    }

    #[test]
    fn parse_mac_address1() {
        let le = LogEntry::new("DATA 127.0.0.1: <14>Dec 12 15:15:20 59-100-240-126.mel.static-ipl.aapt.com.au (\"U7LT,802aa843ac1d,v3.7.21.5389 libubnt[1441]: wevent.ubnt_custom_event(): EVENT_STA_JOIN ath0: 00:34:da:58:8d:a6 / 3");

        assert!(le.parse_mac_address().is_ok());
        assert_eq!(le.parse_mac_address().unwrap(), "00:34:da:58:8d:a6");
    }

    #[test]
    fn parse_mac_address2() {
        let le = LogEntry::new("DATA 127.0.0.1: <14>Dec 12 15:15:20 59-100-240-126.mel.static-ipl.aapt.com.au (\"U7LT,802aa843ac1d,v3.7.21.5389 libubnt[1441]: wevent.ubnt_custom_event(): EVENT_STA_JOIN ath0: 0a:99:da:ab:19:c6 / 3");

        assert!(le.parse_mac_address().is_ok());
        assert_eq!(le.parse_mac_address().unwrap(), "0a:99:da:ab:19:c6");
    }
 
    #[test]
    fn parse_mac_address3() {
        let le = LogEntry::new("[1441]: wevent.ubnt(): ath0: 5a:98:da:ab:19:c6 / 3");

        assert!(le.parse_mac_address().is_ok());
        assert_eq!(le.parse_mac_address().unwrap(), "5a:98:da:ab:19:c6");
    }

    #[test]
    fn forward_die() {
        let le = LogEntry::new("[1441]: wevent.ubnt(): ath0: 5a:98:da:ab:19:c6 / 3");
        let mac = le.parse_mac_address();

        assert!(mac.is_ok());

        let res = le.forward(mac.unwrap(), "doesnotexist.hhd.com.au");

        assert!(res.is_err());
        assert_eq!(res, Err(StatusCode::ServiceUnavailable));
    }

    #[test]
    fn forward_ok() {
        let le = LogEntry::new("DATA 127.0.0.1: <14>Dec 12 15:15:20 59-100-240-126.mel.static-ipl.aapt.com.au (\"U7LT,802aa843ac1d,v3.7.21.5389 libubnt[1441]: wevent.ubnt_custom_event(): EVENT_STA_JOIN ath0: 00:34:da:58:8d:a6 / 3");
        let mac = le.parse_mac_address();

        assert!(mac.is_ok());

        let res = le.forward(mac.unwrap(), "wrestlers.hhd.com.au");

        assert!(res.is_ok());
        assert_eq!(res, Err(StatusCode::Ok));
    }
}
