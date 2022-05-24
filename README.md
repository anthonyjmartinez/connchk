# connchk

## About

`connchk` is command-line network verification utility written in Rust. It aims
to provide a cross platform utility that can verify if your host can reach
targets defined in a TOML document. These hosts are checked in the following
ways:

- For plain TCP hosts, a TcpStream is opened or the relevant error is returned
- For HTTP(S) hosts either
  - A basic check declares success if a status code of 200 is returned
  - A custom check declares success based on a user-defined status code for POSTs of given
    - Form encoded data, or
	- JSON body
  - In either case errors are returned to the user

The application expects exactly one argument which is the TOML document defining
target hosts.

Starting in version 0.5.0, it is also possible to use `connchk` as a Rust library.
Documentation is available [here](https://docs.rs/connchk).

### Install

To get `connchk` run `cargo install connchk` on a system with [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed.

### Example TOML Config
```toml
# example.toml
[[target]]
kind = "Tcp"
desc = "GitLab SSH"
addr = "gitlab.com:22"

[[target]]
kind = "Tcp"
desc = "Freenode IRC"
addr = "irc.freenode.net:6667"

[[target]]
kind = "Tcp"
desc = "httpbin IP endpoint"
addr = "https://httpbin.org/ip"

# Posts as a form and reports success if the status code returned is 400
# which it will be for this bad request to this particular endpoint
[[target]]
kind = "Http"
desc = "httpbin POST endpoint (form)"
addr = "https://httpbin.org/status/undefined"
custom = { params = { someKey = "SpecialValue" }, ok = 400 } 

# Posts as JSON and reports success if the status code returned is 400
# as it will be for this particular endpoint
[[target]]
kind = "Http"
desc = "httpbin JSON endpoint"
addr = "https://httpbin.org/status/400"
custom = { json = { someKey = "SpecialValue" }, ok = 400 } 

# An example failure - this endpoing will return a 502 status code,
# but our configuration expects a 400 
[[target]]
kind = "Http"
desc = "httpbin JSON endpoint - Error"
addr = "https://httpbin.org/status/502"
custom = { json = { someKey = [3, "AnotherValue", false], anotherKey = { nested = "value", count = [1, 2, 3] } }, ok = 400 } 
```

### Example Usage
```
$ ./connchk example.toml 
Successfully connected to GitLab SSH in 72ms
Successfully connected to Freenode IRC in 176ms
Successfully connected to httpbin IP endpoint in 648ms
Successfully connected to httpbin POST endpoint (form) in 666ms
Successfully connected to httpbin JSON endpoint in 647ms
Failed to connect to httpbin JSON endpoint - Error with: 
        Status: 502
        Details:
```

### JSON Bodies

The TOML structure of the configuration file maps on to JSON cleanly. Defining
JSON bodies should be as easy as `custom = { json = <Your JSON Here> }`. While
this was tested to a reasonable degree it's unlikely that every single possibility
has been explored, so if issues are encountered please [let it be known](https://git.staart.one/ajmartinez/connchk/issues).

### Major Changes

- v0.8.0 upgrades argument parsing to `clap` v3.x.
- v0.7.0
  - Changes the project to dual license Apache 2.0 / MIT.
  - Adds a dependencies to [clap](https://crates.io/crates/clap) for argument parsing.
- v0.6.0
  - Refactored away `TcpResource` and `HttpResource` structs differentiating individual `Resource` kinds with the `ResType` enum
  - Added `Resource.kind` to hold `ResType` variants
  - Modified `NetworkResources` to hold `Vec<Resources>` in `NetworkResources.target`
  - Above changes are **BREAKING** with respect to all existing configuration files. Users should:
	- Replace all `[[http]]` or `[[tcp]]` lines with `[[target]]`
	- Add `kind = "Http"` to any block that previously stared with `[[http]]`
	- Add `kind = "Tcp"` to any block that previously stared with `[[tcp]]`
  - Updated dependencies in Cargo.toml & Cargo.lock
  - Added call latency to success output
  - Updated logic to maintain resource order when printing results
- v0.5.0
  - Refactored to produce both binary and library crates
  - Created a common `Resource` struct to map `TcpResource` and `HttpResources` onto for consumption by `par_iter()` by `rayon`
  - The order in which success or failure messages are displayed may not be consistent with the order of definition in the TOML input file
- v0.4.0
  - Adds use of `rayon` to support parallel connection execution
- v0.3.0
  - Adds support for JSON post bodies.
  - Removes declaration of a "bad" status code. Custom tests define only the expected _good_ status code.
- v0.2.1 fixes error handling such that testing does not abort with the first failure
- v0.2.0 disabled the default `reqwest` features to move the package to use of `rustls` instead of `native-tls`


### License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

### Contact

To discuss features, offer assistance, or get help please join the project's [Matrix room](https://matrix.to/#/#connchk:txrx.staart.one).

Copyright (C) 2020-2022 Anthony Martinez
