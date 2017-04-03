# Wrestling Theme Music TCP Adapter

This program will:

  - Listen for rsyslog streams on a given TCP port
  - Attempt to parse out a MAC address and action from the stream
  - Build HTTP POST request to web service, forwarding on details

It runs as a system daemon. So, you can think of it like a proxy that translates ugly syslog messages into web service requests.

This [blog post](https://dev.to/buntine/hulkamania-or-how-i-made-our-office-play-personalized-entrance-theme-music) provides an explanation and video demonstration.

## Process

The configured web service will receive POST requests in the format:
```
https://some.api/<action>/<mac-address>
```

For example:
```
https://my-wifi.com.au/join/12:d4:ab:bd:64:f9
https://my-wifi.com.au/leave/03:ff:cb:34:9a:00
```

## Why?

This allows us to trigger things like:

  - Songs on the Sonos
  - Slack messages on entrance
  - Standup attendance / lateness logging
  - Slack bot to determine if someone is currently in the office

Mapping of MAC address to Human is left to the web service.

## Usage

The binary can be run as:
```
$ wrestlers-adapter [listen_host=127.0.0.1] [listen_port=10514] [forward_host=127.0.0.1] [forward_port=80]
```
