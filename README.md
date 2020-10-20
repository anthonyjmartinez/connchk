# connchk

## About

`connchk` is command-line network checking tool written in Rust. It aims
to provide a cross platform utility that can verify if your host can reach
targets defined in a TOML document. These hosts are checked in the following
ways:

- For plain TCP hosts, a TcpStream is opened or the relevant error is returned
- For HTTP(S) hosts either
  - A basic check declares success if a status code of 200 is returned
  - A custom check declares success and failure with user-defined status codes. This test makes a form-encoded POST request.
  - In either case errors are returned to the user

The application expects exactly one argument which is the TOML document defining
target hosts.

### Install

To get `connchk` run `cargo install connchk` on a system with [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed.

For the time being this is the only way. At a later date packages for various systems may be included in the repository.

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
custom = { params = { someKey = "SpecialValue" }, ok = 400, bad = 403 } 

```

### Example Usage
```
$ ./connchk ~/Projects/connchk/example.toml
Successfully connected to GitLab SSH
Successfully connected to Freenode IRC
Successfully connected to httpbin IP endpoint
Successfully connected to httpbin POST endpoint (form)
```

### TODO

- Add support for testing POST requests with JSON payloads

### License

This project uses [GPL-3.0+](https://www.gnu.org/licenses/gpl-3.0.html).

Copyright (C) 2020 Anthony Martinez
