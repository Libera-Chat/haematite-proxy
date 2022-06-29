# haematite-proxy

A data diode for TS6 server links, allowing a (pseudo)server to link to the network and
receive state updates, without being able to change anything.

`haematite-proxy` can isolate a single server link behind it. On receiving an appropriately
authorised (see below) connection, it will connect to its configured uplink, introduce the
isolated server using the name, SID and password configured in the proxy's config file,
then relay all traffic received from the uplink to the isolated server.

Incoming data from the isolated server towards the network is ignored. After the initial
server introduction and authentication, the only data sent to the uplink is to respond to
pings.

## Building

```
  $ cargo build
```

## Running

```
  $ ./target/debug/haematite-proxy <config filename>
```

## Configuration

See `config/example.conf` in the source repository, however: config is a simple
`key = value` format, with one entry per line. Recognised settings are as follows; all
settings are required unless noted as optional.

server_name
: The server name which the proxy will introduce to the network

sid
: The server ID which the proxy will introduce to the network

uplink_remote_address
: The address of the uplink server

uplink_remote_name
: The name to use when validating the uplink server's TLS certificate. Optional;
  defaults to the value of `uplink_remote_address`.

uplink_remote_port
: The port number to connect to

uplink_password
: The server password to send to the uplink. The corresponding password received from
  the uplink is ignored.

uplink_ca
: The CA certificate file with which to validate the uplink server's TLS certificate.
  Must contain a single certificate in DER format.

ro_listen_address
: The address (IP:port) on which to listen for connections from the isolated read-only
  server.

ro_cert
: The TLS certificate file to use for incoming read-only connections. Must contain a
  single certificate in DER format.

ro_key
: The private key for the certificate in `ro_cert`. Must be in DER format.

auth_ca
: The CA certificate used to authenticate client certificates for incoming connections.
  Must contain a single certificate in DER format.

auth_fingerprint
: The SHA-1 fingerprint of the authorised client certificate for incoming connections.


## Authentication and authorisation

A single instance of `haematite-proxy` supports a single isolated pseudoserver. That
server must connect to the proxy's `ro_listen_address` using TLS, with a client
certificate issued by the CA in `auth_ca`, and whose fingerprint matches
`auth_fingerprint`. There is no further authentication, as no data is read from the
isolated server except to establish and maintain the TLS connection.

When connecting to the uplink, `haematite-proxy` will verify the server's TLS certificate
using the provided `uplink_ca` and server name. It will send the password configured in
`uplink_password`, but will not validate the password received from the uplink server.
That password will be relayed to the isolated server, however; be mindful of this if
using the proxy as a security control.