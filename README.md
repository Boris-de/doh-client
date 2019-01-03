# doh-client
`doh-client` is a DNS over HTTPS client, which opens a local UDP (DNS) port and forwards all DNS queries to a remote
HTTP/2.0 server. By default, the client will connect to the Cloudflare DNS service. It uses [Tokio](https://tokio.rs/)
for all asynchronous IO operations and [Rustls](https://github.com/ctz/rustls) to connect to the HTTPS server.  
[![Build Status](https://travis-ci.org/LinkTed/doh-client.svg?branch=master)](https://travis-ci.org/LinkTed/doh-client)
[![Latest version](https://img.shields.io/crates/v/doh-client.svg)](https://crates.io/crates/doh-client)
[![License](https://img.shields.io/crates/l/doh-client.svg)](https://opensource.org/licenses/BSD-3-Clause)

## Getting Started
`doh-client` is written in Rust. To build it you need the Rust compiler and build system `cargo`.

### Build
```
$ cargo build
```
or to build it as a release build
```
$ cargo build --release
```

### Run
To run the binary, you need one option (see [Options](#Options))
```
$ ./doh-client --cafile /path/to/the/ca/file.pem
```
For example, if you use [Arch Linux](https://www.archlinux.org/) then the following command uses the system cert store:
```
# ./doh-client --cafile /etc/ca-certificates/extracted/tls-ca-bundle.pem
```

#### Linux (`systemd`)
To run the `doh-client` as daemon and without `root` under Linux with `systemd` as init system. The following example 
will connect to the Cloudflare DNS service.
1. Build the binary see [Build](#Build).
2. Copy the binary to `/usr/local/bin` as `root`:
   ```
   # cp target/release/doh-client /usr/local/bin/
   ```
3. Copy the config files to `/etc/systemd/system/` as `root`:
   ```
   # cp doh-client.service doh-client.socket /etc/systemd/system
   ```
   If the location of the binary is different from above then change the path in `doh-client.service` under `ExecStart`. 
   In the config file `doh-client.service` the path of the CA file is set to 
   `/etc/ca-certificates/extracted/tls-ca-bundle.pem`, adjust the path before going further (The path should be correct 
   if you use [Arch Linux](https://www.archlinux.org/)).
4. Reload `systemd` manager configuration:
   ```
   # systemctl daemon-reload
   ```
5. Enable the `doh-client` as a daemon:
   ```
   # systemctl enable doh-client
   ```
6. Reboot the system or start the daemon manually:
   ```
   # systemctl start doh-client
   ```
7. Adjust the `/etc/resolv.conf` by add the following line:
   ```
   nameserver 127.0.0.1
   ```

#### Mac OS (`launchd`)
To run the `doh-client` as daemon and without `root` under Mac OS with `launchd` as init system. The following example 
will connect to the Cloudflare DNS service.
1. Build the binary see [Build](#Build).
2. Copy the binary to `/usr/local/bin` as `root`: 
   ```
   # cp target/release/doh-client /usr/local/bin/
   ```
3. Copy the `launchd` config files to `/Library/LaunchDaemons/` as `root`:
   ```
   # cp com.doh-client.daemon.plist /Library/LaunchDaemons
   ```
   If the location of the binary is different from above then change the path in `com.doh-client.daemon.plist` under 
   `ProgramArguments`. In the config file `com.doh-client.daemon.plist` the path of the CA file is set to 
   `/usr/local/share/doh-client/DigiCert_Global_Root_CA.pem`, download the pem file under the following 
   [link](https://dl.cacerts.digicert.com/DigiCertGlobalRootCA.crt). Before copy the pem file to 
   `/usr/local/share/doh-client/`, make the directory `doh-client` with `mkdir`.
4. Load and start the config file as follow:
   ```
   # launchctl load -w /Library/LaunchDaemons/com.doh-client.daemon.plist
   ```
5. Adjust the `/etc/resolv.conf` by add the following line:
   ```
   nameserver 127.0.0.1
   ```

## Options
`doh-client` has one required option, `--cafile` which sets the path to a pem file, which contains the trusted CA
certificates.
```
$ ./doh-client --help
DNS over HTTPS client 1.2.0
link.ted@mailbox.org
Open a local UDP (DNS) port and forward DNS queries to a remote HTTP/2.0 server.
By default, the client will connect to the Cloudflare DNS service.

USAGE:
    doh-client [FLAGS] [OPTIONS] --cafile <FILE>

FLAGS:
    -g, --get                  Use GET method for the HTTP/2.0 request
    -h, --help                 Prints help information
        --listen-activation    Use file descriptor 3 under Unix as UDP socket or launch_activate_socket() under Mac OS
    -v                         Sets the level of verbosity
    -V, --version              Prints version information

OPTIONS:
    -c, --cafile <FILE>              The path to the pem file, which contains the trusted CA certificates
    -d, --domain <Domain>            The domain name of the remote server [default: cloudflare-dns.com]
    -l, --listen-addr <Addr>         Listen address [default: 127.0.0.1:53]
    -p, --path <STRING>              The path of the URI [default: dns-query]
    -r, --remote-addr <Addr>         Remote address [default: 1.1.1.1:443]
        --retries <UNSIGNED INT>     The number of retries to connect to the remote server [default: 3]
    -t, --timeout <UNSIGNED LONG>    The time in seconds after that the connection would be closed if no response is
                                     received from the server [default: 2]
```
