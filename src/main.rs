#[derive(Debug, Copy, Clone)]
pub struct LogEntry<'a> {
    value: &'a str,
}

impl<'a> LogEntry<'a> {
    fn new(v: &str) -> LogEntry {
        LogEntry{value: v}
    }

    fn parse_mac_address(self) -> Result<&'a str, &'static str> {
        Ok("abc123")
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
        let le = LogEntry::new("test");

        assert_eq!(le.parse_mac_address().unwrap(), "abc123");
    }

    #[test]
    fn forward() {
        let le = LogEntry::new("test");
        let mac = le.parse_mac_address().unwrap();

        assert_eq!(le.forward(mac, "wrestlers.hhd.com.au").unwrap(), 200);
    }
}
