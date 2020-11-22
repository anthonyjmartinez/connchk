# connchk

## About

`connchk` is command-line network checking tool written in Rust. It aims
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

### Install

To get `connchk` run `cargo install connchk` on a system with [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed.

### Example TOML Config
```toml
# example.toml
[[tcp]]
desc = "GitLab SSH"
addr = "gitlab.com:22"

[[tcp]]
desc = "Freenode IRC"
addr = "irc.freenode.net:6667"

[[http]]
desc = "httpbin IP endpoint"
addr = "https://httpbin.org/ip"

# Posts as a form and reports success if the status code returned is 400
# which it will be for this bad request to this particular endpoint
[[http]]
desc = "httpbin POST endpoint (form)"
addr = "https://httpbin.org/status/undefined"
custom = { params = { someKey = "SpecialValue" }, ok = 400 } 

# Posts as JSON and reports success if the status code returned is 400
# as it will be for this particular endpoint
[[http]]
desc = "httpbin JSON endpoint"
addr = "https://httpbin.org/status/400"
custom = { json = { someKey = "SpecialValue" }, ok = 400 } 

# An example failure - this endpoint will return a 502 status code,
# but our configuration expects a 400 
[[http]]
desc = "httpbin JSON endpoint - Error"
addr = "https://httpbin.org/status/502"
custom = { json = { someKey = [3, "AnotherValue", false], anotherKey = { nested = "value", count = [1, 2, 3] } }, ok = 400 }
```

### Example Usage
```
$ ./connchk example.toml 
Successfully connected to GitLab SSH
Successfully connected to Freenode IRC
Successfully connected to httpbin IP endpoint
Successfully connected to httpbin POST endpoint (form)
Successfully connected to httpbin JSON endpoint
Failed to connect to httpbin JSON endpoint - Error with:
        "Failed to connect to httpbin JSON endpoint - Error with: 502, "
```

### JSON Bodies

The TOML structure of the configuration file maps on to JSON cleanly. Defining
JSON bodies should be as easy as `custom = { json = <Your JSON Here> }`. While
this was tested to a reasonable degree it's unlikely that every single possibility
has been explored, so if issues are encountered please [let it be known](https://github.com/anthonyjmartinez/connchk/issues/new/choose).

### Major Changes

- v0.3.0
  - Adds support for JSON post bodies.
  - Removes declaration of a "bad" status code. Custom tests define only the expected _good_ status code.
- v0.2.1 fixes error handling such that testing does not abort with the first failure
- v0.2.0 disabled the default `reqwest` features to move the package to use of `rustls` instead of `native-tls`

### TODO

- Asynchronous connection checking

### License

This project uses [GPL-3.0+](https://www.gnu.org/licenses/gpl-3.0.html).

Copyright (C) 2020 Anthony Martinez
