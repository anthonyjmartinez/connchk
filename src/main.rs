/*   
    connchk gives a status of reachability of plain tcp or http(s) endpoints from your machine
    Copyright (C) 2020-2021 Anthony Martinez

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//!
//! `connchk` is command-line network checking tool written in Rust. It aims
//! to provide a cross platform utility that can verify if your host can reach
//! targets defined in a TOML document.


use std::boxed::Box;
use std::collections::HashMap;
use std::env;
use std::net::{Shutdown, TcpStream};
use rayon::prelude::*;
use reqwest::StatusCode;
use reqwest::blocking::{Client, Response};
use serde::Deserialize;
use serde_json::Value;
use toml;


/// Provides a deserialize target optional parameters for
/// custom HTTP(s) checks.
#[derive(Deserialize, Debug)]
struct HttpOptions {
    params: Option<HashMap<String,String>>,
    json: Option<Value>,
    ok: u16,
}

/// Provides a deserialize target for general parameters
/// for HTTP(s) checks.
#[derive(Deserialize, Debug)]
struct HttpResource {
    desc: String,
    addr: String,
    custom: Option<HttpOptions>,
}

impl HttpResource {
    /// Checks an HTTP(s) endpoint's availability with a GET request.
    /// Prints a success message if the status code is 200 OK, or
    /// failure details in any other case.
    fn check_basic(&self) -> Result<(), Box<dyn std::error::Error>> {
	let client = Client::new();
	let resp = client.get(&self.addr).send()?;
	if resp.status() == StatusCode::OK {
	    Ok(println!("Successfully connected to {}", self.desc))
	} else {
	    let msg = format!("Failed to connect to {} with: {}, {}", self.desc, resp.status().as_str(), resp.text()?);
	    Err(From::from(msg))
	}
    }

    /// Checks an HTTP(s) endpoint's availability with a form POST request.
    /// Values are defined in the `HttpOptions` struct.
    /// Prints a success message if the status code is equal to the `ok` value,
    /// or failure details when the status code is equaly to the `bad` value or
    /// any other value/error.
    fn check_custom(&self, options: &HttpOptions) -> Result<(), Box<dyn std::error::Error>> {

	let client = Client::new();
	let resp: Response;
	if let Some(params) = &options.params {
	    resp = client.post(&self.addr)
		.form(params)
		.send()?;
	    self.custom_resp(options, resp)?
	} else if let Some(json) = &options.json {
	    resp = client.post(&self.addr)
		.json(json)
		.send()?;
	    self.custom_resp(options, resp)?
	};

	Ok(())
    }

    fn custom_resp(&self, options: &HttpOptions, resp: Response) -> Result<(), Box<dyn std::error::Error>> {
	let resp_code = resp.status().as_u16();

	if resp_code == options.ok {
	    Ok(println!("Successfully connected to {}", self.desc))
	} else {
	    let msg = format!("Failed to connect to {} with: {}, {}", self.desc, resp.status().as_str(), resp.text()?);
	    Err(From::from(msg))
	}
    }
}

/// Provides a deserialize target for TCP checks
#[derive(Deserialize, Debug)]
struct TcpResource {
    desc: String,
    addr: String,
}

/// Provides a deserialize target for TOML configuration files
/// defining multiple `TcpResource` or `HttpResource` entities
#[derive(Debug, Deserialize)]
struct NetworkResources {
    http: Option<Vec<HttpResource>>,
    tcp: Option<Vec<TcpResource>>,
}

/// Defines common behavior between `TcpResource` and `HttpResource`
/// structs
trait Checker {
    /// Execute the connection check
    fn check(&self) -> Result<(), Box<dyn std::error::Error>>;

    /// Describe the connection being checked
    fn description(&self) -> &String;
}

impl Checker for HttpResource {
    fn check(&self) -> Result<(), Box<dyn std::error::Error>> {
	match &self.custom {
	    Some(options) => self.check_custom(options),
	    None => self.check_basic(),
	}
    }

    fn description(&self) -> &String {
	&self.desc
    }
}

impl Checker for TcpResource {
    fn check(&self) -> Result<(), Box<dyn std::error::Error>> {
	let stream = TcpStream::connect(&self.addr)?;
	stream.shutdown(Shutdown::Both)?;
	Ok(println!("Successfully connected to {}", self.desc))
    }

    fn description(&self) -> &String {
	&self.desc
    }
}

impl NetworkResources {
    /// Checks all resources contained by a NetworkResources struct.
    /// Any error messages will be printed to the console.
    fn check_resources(self) {
	NetworkResources::check_tcp(self.tcp);
	NetworkResources::check_http(self.http);
    }

    /// Loops through all items present in a NetworkResources.tcp element.
    fn check_tcp(v: Option<Vec<TcpResource>>) {
	if let Some(v) = v {
	    v.par_iter()
		.for_each(|el| match el.check() {
		    Ok(_) => (),
		    Err(e) => println!("Failed to connect to {} with:\n\t{:?}", el.description(), e)
		});
	}
    }
    
    /// Loops through all items present in a NetworkResources.http element.
    fn check_http(v: Option<Vec<HttpResource>>) {
	if let Some(v) = v {
	    v.par_iter()
		.for_each(|el| match el.check() {
		    Ok(_) => (),
		    Err(e) => println!("Failed to connect to {} with:\n\t{:?}", el.description(), e)
		});
	}
    }
}

/// Main entrypoint for connection validation. Once the TOML configuration
/// file has been deserialized all nested `TcpResource` and `HttpResource`
/// targets are checked.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
	return Err(From::from("Pass exactly one argument: the path of a TOML file declaring the network resources to test"))
    } else {
	let config_path = std::path::PathBuf::from(&args[1]);
	let config = std::fs::read_to_string(&config_path)?;
	let resources: NetworkResources = toml::from_str(&config)?;
	resources.check_resources();
    }
   
    Ok(())
}
