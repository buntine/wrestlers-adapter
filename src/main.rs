#[macro_use] extern crate lazy_static;
extern crate regex;

use regex::Regex;

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
            static ref mac: Regex = Regex::new(r"^(:xdigit:{2}::xdigit:{2}::xdigit:{2}::xdigit:{2}::xdigit:{2}::xdigit:{2})$").unwrap();
        }

        let mut cap = mac.captures_iter(&self.value);
        let first = match cap.nth(0) {
            Some(c) => c,
            None => return Err("No mac address present"),
        };

        Ok(first.at(0).unwrap())
    }

    fn forward(self, mac_address: &'a str, host: &'a str) -> Result<i32, i32> {
        Ok(200)
    }
}

fn main() {
    // Run TCP server as daemon on <PORT>
    // Take in string
    // Break into chunks
    // Get MAC address
    // POST request to https://wrestlers.hhd.com.au/<MAC>
    //
    // Also setup to run on bootup
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_log_entry() {
        let le = LogEntry::new("test");

        assert_eq!(le.value, "test");
    }

    #[test]
    fn parse_mac_address() {
        let le = LogEntry::new("DATA 127.0.0.1: <14>Dec 12 15:15:20 59-100-240-126.mel.static-ipl.aapt.com.au (\"U7LT,802aa843ac1d,v3.7.21.5389 libubnt[1441]: wevent.ubnt_custom_event(): EVENT_STA_JOIN ath0: 00:34:da:58:8d:a6 / 3");

        assert_eq!(le.parse_mac_address().unwrap(), "00:34:da:58:8d:a6");
    }

    #[test]
    fn forward() {
        let le = LogEntry::new("test");
        let mac = le.parse_mac_address().unwrap();

        assert_eq!(le.forward(mac, "wrestlers.hhd.com.au").unwrap(), 200);
    }
}
