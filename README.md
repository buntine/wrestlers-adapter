# Wrestling Theme Music TCP Adapter

This programs purpose is to:

  - Listen for rsyslog streams
  - Attempt to parse out a MAC address and action from the stream
  - Build HTTP POST request to API, forwarding on details

It runs as a daemonized TCP server. So, you can think of it like a proxy that translates ugly syslog messages into HTTP API requests.

## Process

The configured web service will receive POST requests in the format:
```
https://some.api/<action>/<mac-address>
```

For example:
```
https://wrestlers.hhd.com.au/join/12:d4:ab:bd:64:f9
https://wrestlers.hhd.com.au/leave/03:ff:cb:34:9a:00
```

## Why?

This allows us to trigger things like:

  - Songs on the Sonos
  - Slack messages on entrance
  - Standup ttendance / lateness logging
  - Slack bot to determine if someone is currently in the office

Mapping of MAC address to Human is left to the API.

## Usage

The binary can be run as:
```
$ wrestlers-adapter [host] [port]
```

Host defaults to **127.0.0.1** and port defaults to **10514**.
