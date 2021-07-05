# Summary
Webserv is a simple, multi-threaded HTTP server with SSL support made with Rust.

# Requirements
Build and run the project using Cargo, Rust's official dependency management and build tool.

```
// Run the program with the -h flag for full usage
cargo build -- args
```

Webserv uses the [openssl crate](https://docs.rs/openssl/0.10.29/openssl/) to decrypt and manage SSL sessions. The openssl crate requires the OpenSSL libraries and headers to build and run webserv (guide available on openssl crate document page [here](https://docs.rs/openssl/0.10.29/openssl/#automatic)).

# Usage

```
USAGE:
    webserv [OPTIONS] <ROOT>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --cert <CERT>    Directory to SSL certificate PEM file. Required if using port 443!
    -k, --key <KEY>      Directory to SSL private key PEM file. Required if using port 443!
    -p, --port <PORT>    Port that the server will bind to.

ARGS:
    <ROOT>    Website root directory.
```

## Example Usage
```
// sudo required for ports below 1024
sudo webserv /path/to/html/root -p 443 -c /path/to/ssl/cert -k /path/to/ssl/private/key

// non-SSL hosting doesn't require CERT or KEY flags!
// default port is 8080 and does not require sudo
webserv /path/to/html/root 

// HINT:
// If your SSL cert and key follow a similar format to `-----BEGIN PRIVATE KEY-----`, then it is already in the PEM format.
// Just make a copy of the cert/key file with the `.pem` file extention added to the end.
```
