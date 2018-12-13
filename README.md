# Interfacing `podman` via the `varlink` protocol

This guide shows how to access podman remotely via the varlink interface with the CLI tools and 
programmatically with python and rust.

Works on Linux, MacOS and Windows 10.

A compatibility matrix can be seen on https://varlink.org/Language-Bindings

> Note: replace 192.168.122.29 in this guide with the IP or hostname of your
> podman machine

## Set up podman on a Fedora/RHEL machine

~~~bash
$ sudo dnf install podman libvarlink-util
$ sudo systemctl enable --now io.podman.socket
$ sudo setfacl -m u:$USER:rx /run/podman
$ sudo setfacl -m u:$USER:rw /run/podman/io.podman
~~~

> Note: Wouldn't it be nice, if there was a podman group owning the socket?

## Windows
Install the OpenSSH Client built by Microsoft in a cmd.exe in admin mode:

~~~cmd
> dism /online /Add-Capability /CapabilityName:OpenSSH.Client~~~~0.0.1.0
~~~

Close cmd.exe window.

> Note: Works also with the ssh of [Git Bash](https://gitforwindows.org/).

### Generate ssh keys
~~~bash
$ ssh-keygen
~~~

Optionally, if you don't want to enter your password every time:
~~~bash
$ ssh-copy-id 192.168.122.29
~~~

## Python

### Install Python
https://www.python.org/downloads/

### Install varlink for Python

~~~bash
$ pip install varlink
~~~

### Test if the varlink cli module works

~~~bash
$ python -m varlink.cli --help
usage: cli.py [-h] [-r RESOLVER] [-A ACTIVATE] [-b BRIDGE]
              {info,help,bridge,call} ...
…
~~~

### Port Forwarding

Open a ssh session with port forwarding of the podman unix domain socket to a local TCP socket:

~~~bash
$ ssh -L 127.0.0.1:1234:/run/podman/io.podman 192.168.122.29
~~~

> Note: only required for the python part.

### Interfacing podman with the python cli module

From a different window:

~~~bash
$ python -m varlink.cli info tcp:127.0.0.1:1234
Vendor: Atomic
Product: podman
Version: 0.10.1
URL: https://github.com/containers/libpod
Interfaces:
   org.varlink.service
   io.podman

$ python -m varlink.cli call tcp:127.0.0.1:1234/io.podman.Ping {}
{
  "ping": {
    "message": "OK"
  }
}
~~~

### Using varlink with python programmatically
~~~bash
$ python
Python 3.7.1 (v3.7.1:260ec2c36a, Oct 20 2018, 14:57:15) [MSC v.1915 64 bit (AMD64)] on win32
Type "help", "copyright", "credits" or "license" for more information.
>>> import varlink
>>> client = varlink.Client("tcp:127.0.0.1:1234")
>>> podman = client.open("io.podman")
>>> podman.Ping()
{'ping': {'message': 'OK'}}
>>> podman.GetInfo()
{'info': {'host': {'buildah_version': '1.5-dev', 'distribution': {'distribution': 'fedora', 'version': '29'}, 'mem_free': 2158669824, 'mem_total': 4133470208, 'swap_free': 4269797376, 'swap_total': 0, 'arch': 'amd64', 'cpus': 2, 'hostname': 'FedVM-29', 'kernel': '4.18.17-300.fc29.x86_64', 'os': 'linux', 'uptime': '50h 22m 0.38s (Approximately 2.08 days)'}, 'registries': ['docker.io', 'registry.fedoraproject.org', 'quay.io', 'registry.access.redhat.com', 'registry.centos.org'], 'insecure_registries': [], 'store': {'containers': 0, 'images': 0, 'graph_driver_name': 'overlay', 'graph_driver_options': 'overlay.mountopt=nodev, overlay.override_kernel_check=true', 'graph_root': '/var/lib/containers/storage', 'graph_status': {'backing_filesystem': 'extfs', 'native_overlay_diff': 'true', 'supports_d_type': 'true'}, 'run_root': '/var/run/containers/storage'}, 'podman': {'compiler': 'gc', 'go_version': 'go1.11', 'podman_version': '', 'git_commit': ''}}}
>>> podman.GetVersion()
{'version': {'version': '0.10.1', 'go_version': 'go1.11', 'git_commit': '', 'built': 0, 'os_arch': 'linux/amd64'}}
>>> info=podman.GetInfo()
>>> print(info["info"]["host"]["uptime"])
50h 23m 8.84s (Approximately 2.08 days)
>>> print(info["info"]["host"]["os"])
linux
>>>
~~~

## Rust

### Install the rust toolchain

#### Windows
First install the C++ part of https://visualstudio.microsoft.com/downloads/

#### All
https://rustup.rs/

### Install varlink-cli

#### For non-Linux systems:

~~~bash
$ cargo install varlink-cli
~~~

> Note: Ensure that $HOME/.cargo/bin is in your PATH or copy $HOME/.cargo/bin/varlink
> in one of your path directories  

#### For Linux systems:

You can also use `varlink` util from [libvarlink](https://github.com/varlink/libvarlink)
or install `libvarlink-util` on Fedora/RHEL machines.

### Running varlink-cli
Without an open ssh connection like in the python case, the rust version can use the `--bridge` feature.

~~~bash
$ varlink --bridge "ssh 192.168.122.29 varlink bridge --connect unix:/run/podman/io.podman" info
Vendor: Atomic
Product: podman
Version: 0.10.1
URL: https://github.com/containers/libpod
Interfaces:
  org.varlink.service
  io.podman


$ varlink --bridge "ssh 192.168.122.29 varlink bridge --connect unix:/run/podman/io.podman" call io.podman.Ping
{
  "ping": {
    "message": "OK"
  }
}
~~~

### Create a rust application

Either clone this [repository](https://github.com/haraldh/podmanrs) or:

~~~bash
$ cargo new --bin podmanrs
$ cd podmanping
~~~

Download the varlink interface from the running podman varlink service:
 
~~~bash
$ varlink --bridge "ssh 192.168.122.29 varlink bridge --connect unix:/run/podman/io.podman" help io.podman > src/io.podman.varlink
~~~

create `build.rs`:
~~~rust
extern crate varlink_generator;

fn main() {
   varlink_generator::cargo_build_tosource("src/io.podman.varlink", true);
}
~~~

create `Cargo.toml`:
~~~toml
[package]
name = "podmanrs"
version = "0.1.0"
authors = ["Harald Hoyer <harald@redhat.com>"]
build = "build.rs"
edition = "2018"

[dependencies]
varlink = "5.3"
serde = "1"
serde_derive = "1"
serde_json = "1"
failure_derive = "0.1"
failure = "0.1"

[build-dependencies]
varlink_generator = "6"
~~~

create `src/main.rs`:
~~~rust
mod io_podman;

use varlink::Connection;
use crate::io_podman::*;

fn main() -> Result<()> {
  let connection =
      Connection::with_bridge("ssh 192.168.122.29 -- varlink bridge --connect unix:/run/podman/io.podman")?;
  let mut iface = VarlinkClient::new(connection.clone());
  let reply = iface.ping().call()?;
  println!("Ping() replied with '{}'", reply.ping.message);
  let reply = iface.get_info().call()?;
  println!("Hostname: {}", reply.info.host.hostname);
  println!("Info: {:#?}", reply.info);
  Ok(())
}
~~~

Now run it:

~~~bash
$ cargo run
~~~

Have fun!

-- Harald Hoyer <harald@redhat.com>