# Installation
install with cargo: `cargo install --git https://github.com/Vescusia/dl_redirector.git`

or just download prebuild binaries from the [releases](https://github.com/Vescusia/dl_redirector/releases)

# Usage
## Receiver
Simply receive a file with
`dl_receiver myfriendsdomain.com`

or specify a download directory with
`dl_receiver myfriendsdomain.com -d /path/to/download/dir`

## Redirector
Simply redirect a file with 
`dl_redirector url.to/redirect.file`

The default socket is 0.0.0.0:4444.

Use `dl_redirector url.to/redirect.file -s <address>:<port>` to configure your socket

### Open/Forward Ports
To redirect traffic, you will have to open and forward the matching port on both your router and machine.
