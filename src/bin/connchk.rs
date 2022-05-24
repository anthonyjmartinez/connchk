/*   
    connchk gives a status of reachability of plain tcp or http(s) endpoints from your machine
    Copyright (C) 2020-2022 Anthony Martinez

    Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
    http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
    http://opensource.org/licenses/MIT>, at your option. This file may not be
    copied, modified, or distributed except according to those terms.
*/

use connchk::{arg_handler, NetworkResources};

/// Main entrypoint for connection validation. Once the TOML configuration
/// file has been deserialized all nested `TcpResource` and `HttpResource`
/// targets are checked.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(config_path) = arg_handler() {
	let config = std::fs::read_to_string(&config_path)?;
	let mut resources: NetworkResources = toml::from_str(&config)?;
	resources.check_resources();
    }
   
    Ok(())
}
