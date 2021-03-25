/*   
    connchk gives a status of reachability of plain tcp or http(s) endpoints from your machine
    Copyright (C) 2020-2021 Anthony Martinez

    Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
    http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
    http://opensource.org/licenses/MIT>, at your option. This file may not be
    copied, modified, or distributed except according to those terms.
*/

//!
//! `connchk` is command-line network checking tool written in Rust. It aims
//! to provide a cross platform utility that can verify if your host can reach
//! targets defined in a TOML document. Using the library a user can incorporate
//! network checks into independent works.

use std::boxed::Box;
use std::collections::HashMap;
use std::net::{Shutdown, TcpStream};
use std::path::PathBuf;
use std::time::Instant;

use clap::{App, Arg};
use rayon::prelude::*;
use reqwest::StatusCode;
use reqwest::blocking::{Client, Response};
use serde::Deserialize;
use serde_json::Value;


/// Provides argument handling using Clap
pub fn arg_handler() -> Option<PathBuf> {
    let matches = App::new("connchk")
        .version("0.7.0")
        .author("Anthony Martinez <anthony@ajmartinez.com>")
	.about("Command-line network checking tool written in Rust")
        .arg(Arg::with_name("config")
             .help("Path to the configuration file to use")
             .index(1)
             .required(true))
        .get_matches();
	
    if let Some(conf_path) = matches.value_of("config") {
	Some(PathBuf::from(conf_path))
    } else {
	None
    }
}

/// Provides a deserialize target for optional parameters in
/// custom HTTP(s) checks.
#[derive(Deserialize, Debug, Clone)]
pub struct HttpOptions {
    pub params: Option<HashMap<String,String>>,
    pub json: Option<Value>,
    pub ok: u16,
}

/// A generic resource combining all possible fields into a common type
#[derive(Deserialize, Debug)]
pub struct Resource {
    pub desc: String,
    pub addr: String,
    pub custom: Option<HttpOptions>,
    pub kind: ResType,
    pub res: Option<String>,
}

impl Resource {
    /// Executes connectivity checks for each type defined in [`ResType`]
    pub fn check(&self) -> Result<(), Box<dyn std::error::Error>> {
	match self.kind {
	    ResType::Tcp => {
		self.check_tcp()?;
	    },
	    ResType::Http => {
		if let Some(opts) = &self.custom {
		    self.check_http_custom(&opts)?;
		} else {
		    self.check_http_basic()?;
		}
	    }
	}
	Ok(())
    }

    /// Checks an HTTP(s) endpoint's availability with a GET request.
    /// Prints a success message if the status code is 200 OK, or
    /// failure details in any other case.
    fn check_http_basic(&self) -> Result<(), Box<dyn std::error::Error>> {
	let client = Client::new();
	let resp = client.get(&self.addr).send()?;
	if resp.status() == StatusCode::OK {
	    Ok(())
	} else {
	    let msg = format!("\n\tStatus: {}\n\tDetails: {}", resp.status().as_str(), resp.text()?);
	    Err(From::from(msg))
	}
    }

    /// Checks an HTTP(s) endpoint's availability with a form POST request.
    /// Values are defined in the `HttpOptions` struct.
    /// Prints a success message if the status code is equal to the `ok` value,
    /// or failure details when the status code is equaly to the `bad` value or
    /// any other value/error.
    fn check_http_custom(&self, options: &HttpOptions) -> Result<(), Box<dyn std::error::Error>> {
	let client = Client::new();
	let resp: Response;
	if let Some(params) = &options.params {
	    resp = client.post(&self.addr)
		.form(params)
		.send()?;
	    self.custom_http_resp(options, resp)?
	} else if let Some(json) = &options.json {
	    resp = client.post(&self.addr)
		.json(json)
		.send()?;
	    self.custom_http_resp(options, resp)?
	};

	Ok(())
    }

    /// Returns the response details for HTTP(s) checks when the [`HttpResource.custom`] field
    /// is used. 
    fn custom_http_resp(&self, options: &HttpOptions, resp: Response) -> Result<(), Box<dyn std::error::Error>> {
	let resp_code = resp.status().as_u16();
	if resp_code == options.ok {
	    Ok(())
	} else {
	    let msg = format!("\n\tStatus: {}\n\tDetails: {}", resp.status().as_str(), resp.text()?);
	    Err(From::from(msg))
	}
    }

    /// Checks a TCP endpoint's availability with by establishing a [`TcpStream`]
    /// Prints a success message if the stream opens without error, or returns
    /// failure details in any other case.
    fn check_tcp(&self) -> Result<(), Box<dyn std::error::Error>> {
	let stream = TcpStream::connect(&self.addr)?;
	stream.shutdown(Shutdown::Both)?;
	Ok(())
    }
}

/// Classifies the resource type for the top-level [`Resource`] struct
#[derive(Deserialize, Debug)]
pub enum ResType {
    /// An HTTP(s) resource
    Http,
    /// A TCP resource
    Tcp,
}

/// Provides a deserialize target for TOML configuration files
/// defining multiple [`Resource`] entities
#[derive(Deserialize, Debug)]
pub struct NetworkResources {
    pub target: Vec<Resource>,
}

impl NetworkResources {
    /// Executes parallel connectivity checks for all [`Resource`]
    /// objects contained within the higher level [`NetworkResources`]
    /// struct. Prints success message with call latency or failure message
    /// with available details. Maintains the resource order defined in the
    /// supplied TOML configuration file.
    pub fn check_resources(&mut self) {
	self.target.par_iter_mut()
	    .for_each(|el| {
		let now = Instant::now();
		match el.check() {
		    Ok(_) => {
			let dur = now.elapsed().as_millis();
			let res = format!("Successfully connected to {} in {}ms", el.desc, dur);
			el.res = Some(res);
		    },
		    Err(e) => {
			let res = format!("Failed to connect to {} with: {}", el.desc, e);
			el.res = Some(res);
		    }
		}
	    });

	for target in self.target.iter() {
	    if let Some(result) = &target.res {
		println!("{}", result)
	    }
	}
    }
}
