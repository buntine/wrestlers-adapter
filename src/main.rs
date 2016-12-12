struct LogEntry<'a> {
    value: &'a str,
}

impl<'a> LogEntry<'a> {
    fn new(v: &str) -> LogEntry {
        LogEntry{value: v}
    }

    fn parse_mac_address(self) -> Result<&'a str, &'static str> {
        Ok("abc123")
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

    let le = LogEntry::new("test test");
    println!("{}", le.parse_mac_address().unwrap());
}
