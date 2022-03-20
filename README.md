# argv_to_mqtt
A simple program to push arbitrary MQTT events from arguments, written in Rust.

## Why?

At home, I run a DHCP server using Ubuntu. For security reasons, I wanted to be informed whenever an IP address is given to an unknown MAC address. It turns out ISC DHCP shows supports events ([https://kb.isc.org/docs/isc-dhcp-44-manual-pages-dhcpdconf](https://kb.isc.org/docs/isc-dhcp-44-manual-pages-dhcpdconf)). Basically on commit, release and expiry of an address, ISC DHCP Server can execute an arbitrary command. 
I wanted to have the DHCP server push the event to a MQTT server in order to decouple the message publishing to its handling. 
Hence this super simple program: all it does it to gather the command line arguments an it pushes them to a MQTT server.

## How

This program simply read all its command line parameters, constructs a topic using the first one (excluding its program name of course) and pushes a message with a JSON-formatted string array with the remaining arguments. So, for example:

```bash
argv_to_mqtt dhcp/commit my first example
```

creates this message:

```
argv_to_mqtt/dhcp/commit ["my","first","example"]
```

As shown by `mosquitto_sub` in verbose mode.

#### DHCP

In my case, configuring my ISC DHCP Server adding these lines:

```
on commit {                                              
  set clip = binary-to-ascii(10, 8, ".", leased-address);           
  set clhw = binary-to-ascii(16, 8, ":", substring(hardware, 1, 6));                      
  execute("/usr/local/bin/argv_to_mqtt", "dhcp/commit/declared", clip, clhw, host-decl-name);
  execute("/usr/local/bin/argv_to_mqtt", "dhcp/commit", clip, clhw);
}
```

Gives these events:

```
argv_to_mqtt/dhcp/commit/declared ["10.100.6.189","fc:f5:c4:f:eb:84","living-room.inkplate.display"]
argv_to_mqtt/dhcp/commit ["10.100.6.189","fc:f5:c4:f:eb:84"]
```

## Configuration

The program needs a configuration file (see [sample_config.toml](https://github.com/MindFlavor/argv_to_mqtt/blob/main/sample_config.toml)) to find the MQTT Server and port. The file can either be `~/.config/argv_to_mqtt/config.toml` or `/etc/argv_to_mqtt/config.toml`; the former, if found, takes precedence.

## How to build

You need a recent Rust toolchain for this. Just clone the repo, go inside the folder and issue:

```bash
cargo install --path .
```
