# Jaelegram - Peer to Peer Messaging 📬
A peer to peer messenger focused on security of messages and reliability of deliverability. This software is built specifically to combat situations where we expect frequent disconnects from the network. While there is a central server that servers as an entry point into a given rendition of the network, the work that this node does is minimized.

The goals of this project are to minimize the number of nodes that a message passes through in expectation and trying to ensure that no single node in the network (including the server) has the ability to see all the messages. The language of choice for this project is Rust as this language has robust paralle processing integration as well as straightforward string matching and parsing capabilities. 

## Usage

This project does not require any special rust tools, just [Rust](https://www.rust-lang.org/tools/install).

### Server
The server in this project serves purely as an entry point. In further iterations of this project, the server becomes less and less important in terms of the number of purposes it serves. The various verisons of the client can be toggled to see these changes - the server will maintain the ability to serve the requests, but does not do so in later versions. The server can be run simply by entering the `messaging_server` crate (directory) and running

```
cargo run
```

This will take a while to build as it requires the requisite packages, but once running should just print logs of messages. There is no command line interactions with the server.

### Clients

The client comes in a few different versions - they can all be found in `./bins/` and run from there. They differ mainly in how they send the messages and how much work they do. They can all be used with the main server, but will allow for a tradeoff of performance vs security.
