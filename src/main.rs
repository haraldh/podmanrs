mod io_podman;

use crate::io_podman::*;
use varlink::Connection;

fn main() -> Result<()> {
    let connection = Connection::with_bridge(
        "ssh 192.168.122.29 varlink bridge --connect unix:/run/podman/io\
         .podman",
    )?;
    let mut iface = VarlinkClient::new(connection.clone());
    let reply = iface.ping().call()?;
    println!("Ping() replied with '{}'", reply.ping.message);
    let reply = iface.get_info().call()?;
    println!("Hostname: {}", reply.info.host.hostname);
    println!("Info: {:#?}", reply.info);
    Ok(())
}
