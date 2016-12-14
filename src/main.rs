#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate regex;
extern crate hyper;
extern crate daemonize;

use regex::Regex;

use std::io::prelude::*;
use std::net::{TcpListener};
use std::env;

use hyper::client::Client;
use hyper::status::StatusCode;

use daemonize::Daemonize;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Action<'a> {
    Join(&'a str),
    Leave(&'a str),
}

impl<'a> Action<'a> {
    fn from_str(s: &'a str, mac: &'a str) -> Result<Action<'a>, &'static str> {
        match s {
            "JOIN" => Ok(Action::Join(mac)),
            "LEAVE" => Ok(Action::Leave(mac)),
            _ => Err("Invalid action")
        }
    }

    fn to_url(self, host: &'a str) -> String {
        let (name, mac) = match self {
            Action::Join(m) => ("join", m),
            Action::Leave(m) => ("leave", m),
        };

        format!("http://{}/{}/{}", host, name, mac)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LogEntry<'a> {
    value: &'a str,
}

impl<'a> LogEntry<'a> {
    fn new(v: &str) -> LogEntry {
        LogEntry{value: v}
    }

    fn parse_action(self) -> Result<Action<'a>, &'static str> {
        lazy_static! {
            static ref MAC: Regex = Regex::new(r"(?P<action>JOIN|LEAVE).+(?P<mac>[0-9a-f]{2}:[0-9a-f]{2}:[0-9a-f]{2}:[0-9a-f]{2}:[0-9a-f]{2}:[0-9a-f]{2})").unwrap();
        }

        let mut cap = MAC.captures_iter(&self.value);
        let first = cap.nth(0).ok_or("Invalid log entry format")?;
        let action = first.name("action").ok_or("Invalid log entry format")?;

        match first.name("mac") {
            Some(m) => Action::from_str(&action, &m),
            None => Err("Invalid log entry format"),
        }
    }

    fn forward(self, action: &Action, host: &'a str) -> Result<StatusCode, StatusCode> {
        let client = Client::new();
        let url = action.to_url(host);

        match client.post(&url[..]).send() {
            Ok(r) => Ok(r.status),
            Err(_) => Err(StatusCode::ServiceUnavailable),
        }
    }
}

fn main() {
    let mut args = env::args().skip(1);
    let host = args.next().unwrap_or("127.0.0.1".to_string());
    let port = args.next().unwrap_or("10514".to_string());
    let socket = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&socket[..]).expect(&format!("Cannot establish connection on {}", socket));

    env_logger::init().expect("Cannot open log.");

    info!("Starting daemon on {}", socket);

    let daemonize = Daemonize::new()
        .pid_file("/tmp/wresters-adapter.pid")
        .chown_pid_file(true);

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

                let action = match le.parse_action() {
                    Ok(a) => a,
                    Err(e) => {
                        error!("Failed: {}, {}", e, stream);
                        continue;
                    },
                };

                match le.forward(&action, "127.0.0.1:8000") {
                    Ok(_) => info!("Sent: {:?}", action),
                    Err(_) => warn!("Failed: {:?}", action),
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
    fn parse_action_die() {
        let le = LogEntry::new("this is invalid");

        assert!(le.parse_action().is_err());
        assert_eq!(le.parse_action(), Err("Invalid log entry format"));
    }

    #[test]
    fn parse_action1() {
        let le = LogEntry::new("DATA 127.0.0.1: <14>Dec 12 15:15:20 10.our.host.com.au (\"YO,v3.7.21.5389 libubnt[1441]: wevent.cust(): EVENT_STA_JOIN ath0: 00:34:da:58:9d:a7 / 3");

        assert!(le.parse_action().is_ok());
        assert_eq!(le.parse_action().unwrap(), Action::Join("00:34:da:58:9d:a7"));
    }

    #[test]
    fn parse_action2() {
        let le = LogEntry::new("DATA 127.0.0.1: <14>Dec 12 15:15:20 10.our.host.com.au (\"YO,v3.7.21.5389 libubnt[1441]: wevent.cust(): EVENT_STA_JOIN ath0: 0a:99:da:ab:19:c6 / 3");

        assert!(le.parse_action().is_ok());
        assert_eq!(le.parse_action().unwrap(), Action::Join("0a:99:da:ab:19:c6"));
    }
 
    #[test]
    fn parse_action3() {
        let le = LogEntry::new("[1441]: wevent.ubnt(): EVENT_STA_LEAVE ath0: 5a:98:da:ab:19:c6 / 3");

        assert!(le.parse_action().is_ok());
        assert_eq!(le.parse_action().unwrap(), Action::Leave("5a:98:da:ab:19:c6"));
    }

    #[test]
    fn forward_die() {
        let le = LogEntry::new("[1441]: EVENT_JOIN wevent.ubnt(): ath0: 5a:98:da:ab:19:c6 / 3");
        let action = le.parse_action();

        assert!(action.is_ok());

        let res = le.forward(&action.unwrap(), "doesnotexist.hhd.com.au");

        assert!(res.is_err());
        assert_eq!(res, Err(StatusCode::ServiceUnavailable));
    }

    #[test]
    fn forward_ok() {
        let le = LogEntry::new("DATA 127.0.0.1: <14>Dec 12 15:15:20 10.our.host.com.au (\"YO,v3.7.21.5389 libubnt[1441]: wevent.cust(): EVENT_STA_JOIN ath0: 00:34:da:58:8d:a6 / 3");
        let action = le.parse_action();

        assert!(action.is_ok());

        let res = le.forward(&action.unwrap(), "wrestlers.hhd.com.au");

        assert!(res.is_ok());
        assert_eq!(res, Err(StatusCode::Ok));
    }

    #[test]
    fn action_from_str_ok() {
        let j = Action::from_str("JOIN", "123");
        assert_eq!(j, Ok(Action::Join("123")));

        let l = Action::from_str("LEAVE", "123");
        assert_eq!(l, Ok(Action::Leave("123")));
    }

    #[test]
    fn action_from_str_die() {
        let j = Action::from_str("UNKNOWN", "123");
        assert_eq!(j, Err("Invalid action"));
    }

    #[test]
    fn action_to_url() {
        let j = Action::from_str("JOIN", "123").unwrap();
        assert_eq!(j.to_url("test.com"), "http://test.com/join/123");

        let l = Action::from_str("LEAVE", "12:32:45:65:aa:ff").unwrap();
        assert_eq!(l.to_url("tester.com"), "http://tester.com/leave/12:32:45:65:aa:ff");
    }
}
