mod io_podman;

use crate::io_podman::*;
use varlink::Connection;
use std::result::Result;
use std::error::Error;

fn main() -> Result<(), Box<Error>> {
    let connection = Connection::with_bridge(
        "ssh -T <podman-machine> -- varlink bridge --connect unix:/run/podman/io.podman",
    )?;
    let mut podman = VarlinkClient::new(connection.clone());
    let reply = podman.ping().call()?;
    println!("Ping() replied with '{}'", reply.ping.message);
    let reply = podman.get_info().call()?;
    println!("Hostname: {}", reply.info.host.hostname);
    println!("Info: {:#?}", reply.info);
    Ok(())
}
