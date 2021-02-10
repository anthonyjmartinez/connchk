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
//! targets defined in a TOML document. Using the library a user can incorporate
//! network checks into independent works.

use std::boxed::Box;
use std::collections::HashMap;
use std::net::{Shutdown, TcpStream};
use rayon::prelude::*;
use reqwest::StatusCode;
use reqwest::blocking::{Client, Response};
use serde::Deserialize;
use serde_json::Value;


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

    /// Returns the description of the [`Resource`]
    pub fn description(&self) -> &String {
	&self.desc
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
    /// struct.
    pub fn check_resources(self) {
	self.target.par_iter()
	    .for_each(|el| match el.check() {
		Ok(_) => println!("Successfully connected to {}", el.description()),
		Err(e) => println!("Failed to connect to {} with: {}", el.description(), e)
	    });
	
    }
}
