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

use std::env;

use connchk::NetworkResources;
use toml;

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
