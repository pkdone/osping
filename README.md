# os-ping

A Rust example application which uses the `ping` executable available in the underlying host OS, to test that a remote host can be contacted over the network with an _ICMP Ping_ raw socket connection. This technique is used as it does not require the elevated privileges that would need to be given to the Rust application to open a raw socket for ICMP directly to a host.

# Tested Operating Systems

* Ubuntu 20.04 (x86-64)

