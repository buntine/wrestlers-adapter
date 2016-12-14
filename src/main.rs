#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate regex;
extern crate hyper;
extern crate daemonize;

use regex::Regex;

use std::io::prelude::*;

use std::net::{TcpListener};

use hyper::client::Client;
use hyper::status::StatusCode;

use daemonize::Daemonize;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Action<'a> {
    Join(&'a str),
    Leave(&'a str),
}

#[derive(Debug, Copy, Clone)]
pub struct LogEntry<'a> {
    value: &'a str,
}

impl<'a> LogEntry<'a> {
    fn new(v: &str) -> LogEntry {
        LogEntry{value: v}
    }

    fn parse_mac_address(self) -> Result<Action<'a>, &'static str> {
        lazy_static! {
            static ref MAC: Regex = Regex::new(r"([0-9a-f]{2}:[0-9a-f]{2}:[0-9a-f]{2}:[0-9a-f]{2}:[0-9a-f]{2}:[0-9a-f]{2})").unwrap();
        }

        let mut cap = MAC.captures_iter(&self.value);
        let first = match cap.nth(0) {
            Some(c) => c,
            None => return Err("No mac address present"),
        };

        match first.at(0) {
            Some(m) => Ok(Action::Join(m)),
            None => Err("No mac address present"),
        }
    }

    fn forward(self, mac_address: &Action, host: &'a str) -> Result<StatusCode, StatusCode> {
        let client = Client::new();
        let url = match mac_address {
            &Action::Join(m) => format!("http://{}/join/{}", host, m),
            &Action::Leave(m) => format!("http://{}/leave/{}", host, m),
        };

        match client.post(&url[..]).send() {
            Ok(r) => Ok(r.status),
            Err(_) => Err(StatusCode::ServiceUnavailable),
        }
    }
}

fn main() {
    const HOST: &'static str = "127.0.0.1";
    const PORT: i32 = 10514;
    const USER: &'static str = "andrew";
    const GROUP: &'static str = "andrew";

    let socket = format!("{}:{}", HOST, PORT);
    let listener = TcpListener::bind(&socket[..]).expect(&format!("Cannot establish connection on {}", socket));

    env_logger::init().expect("Cannot open log.");

    info!("Starting daemon");

    let daemonize = Daemonize::new()
        .pid_file("/tmp/wresters-adapter.pid")
        .chown_pid_file(true)
        .user(USER)
        .group(GROUP);

    match daemonize.start() {
        Ok(_) => info!("Success, daemonized"),
        Err(e) => error!("{}", e),
    };

    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                let mut stream = String::new();

                if !s.read_to_string(&mut stream).is_ok() {
                    error!("Failed: Invalid stream");
                    continue;
                }

                let le = LogEntry::new(&stream[..]);

                let mac = match le.parse_mac_address() {
                    Ok(a) => a,
                    Err(e) => {
                        error!("Failed: {}, {}", e, stream);
                        continue;
                    },
                };

                match le.forward(&mac, "127.0.0.1") {
                    Ok(_) => info!("Sent: {:?}", mac),
                    Err(_) => warn!("Failed: {:?}", mac),
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
        let le = LogEntry::new("DATA 127.0.0.1: <14>Dec 12 15:15:20 10.our.host.com.au (\"YO,v3.7.21.5389 libubnt[1441]: wevent.cust(): EVENT_STA_JOIN ath0: 00:34:da:58:9d:a7 / 3");

        assert!(le.parse_mac_address().is_ok());
        assert_eq!(le.parse_mac_address().unwrap(), Action::Join("00:34:da:58:9d:a7"));
    }

    #[test]
    fn parse_mac_address2() {
        let le = LogEntry::new("DATA 127.0.0.1: <14>Dec 12 15:15:20 10.our.host.com.au (\"YO,v3.7.21.5389 libubnt[1441]: wevent.cust(): EVENT_STA_JOIN ath0: 0a:99:da:ab:19:c6 / 3");

        assert!(le.parse_mac_address().is_ok());
        assert_eq!(le.parse_mac_address().unwrap(), Action::Join("0a:99:da:ab:19:c6"));
    }
 
    #[test]
    fn parse_mac_address3() {
        let le = LogEntry::new("[1441]: wevent.ubnt(): EVENT_STA_LEAVE ath0: 5a:98:da:ab:19:c6 / 3");

        assert!(le.parse_mac_address().is_ok());
        assert_eq!(le.parse_mac_address().unwrap(), Action::Leave("5a:98:da:ab:19:c6"));
    }

    #[test]
    fn forward_die() {
        let le = LogEntry::new("[1441]: wevent.ubnt(): ath0: 5a:98:da:ab:19:c6 / 3");
        let mac = le.parse_mac_address();

        assert!(mac.is_ok());

        let res = le.forward(&mac.unwrap(), "doesnotexist.hhd.com.au");

        assert!(res.is_err());
        assert_eq!(res, Err(StatusCode::ServiceUnavailable));
    }

    #[test]
    fn forward_ok() {
        let le = LogEntry::new("DATA 127.0.0.1: <14>Dec 12 15:15:20 10.our.host.com.au (\"YO,v3.7.21.5389 libubnt[1441]: wevent.cust(): EVENT_STA_JOIN ath0: 00:34:da:58:8d:a6 / 3");
        let mac = le.parse_mac_address();

        assert!(mac.is_ok());

        let res = le.forward(&mac.unwrap(), "wrestlers.hhd.com.au");

        assert!(res.is_ok());
        assert_eq!(res, Err(StatusCode::Ok));
    }
}
