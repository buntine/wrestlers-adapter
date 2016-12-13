# Wrestling Theme Music TCP Adapter

This programs purpose is to:

  - Listen for rsyslog streams
  - Attempt to parse out a MAC address from the stream
  - Build HTTP POST request to API, forwarding on MAC address

It runs as a daemonized TCP server. So, you can think of it like a proxy that translates ugly syslog messages into HTTP API requests.

## Process

The configured web service will receive POST requests in the format:
```
https://some.api/<mac-address>
```

For example:
```
https://wrestlers.hhd.com.au/12:d4:ab:bd:64:f9
```

## Why?

This allows us to trigger things like:

  - Songs on the Sonos
  - Slack messages
  - Attendance / lateness logging

Mapping of MAC address to Human is left to the API.
