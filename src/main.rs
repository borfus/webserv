use webserv::ThreadPool;
use std::io::prelude::*;
use std::net::{TcpListener};
use std::{fs, env, process, str};
use std::path::Path;
use std::vec::Vec;
use std::sync::Arc;
use std::time::SystemTime;

use openssl::ssl::{SslMethod, SslAcceptor, SslFiletype};

#[macro_use]
extern crate clap;
use clap::ArgMatches;

extern crate chrono;
use chrono::offset::Utc;
use chrono::DateTime;

struct Config {
    root: String,
    port: String,
    key: Option<String>,
    cert: Option<String>
}

impl Config {
    fn new(args: ArgMatches) -> Result<Config, &str> {
        let root = match args.value_of("ROOT") {
            Some(root) => root,
            None => panic!("Error obtaining ROOT argument")
        };
        let root = String::from(root);

        let port = match args.value_of("PORT") {
            Some(port) => port,
            None => "8080"
        };
        let port = String::from(port);

        let key = match args.value_of("KEY") {
            Some(key) => Some(String::from(key)),
            None if port == "443" => {
                panic!("Command line argument -k or --key is required if using port 443!");
            },
            None => None
        };

        let cert = match args.value_of("CERT") {
            Some(cert) => Some(String::from(cert)),
            None if port == "443" => {
                panic!("Command line argument -c or --cert is required if using port 443!");
            },
            None => None
        };

        Ok(Config{root, port, key, cert})
    }
}

fn main() {
    let args = clap_app!(webserv =>
        (version: "1.0")
        (author: "Peter Bell")
        (about: "Basic HTTP Server")
        (@arg PORT: -p --port +takes_value "Port that the server will bind to.")
        (@arg KEY: -k --key +takes_value "Directory to SSL private key PEM file. Required if using port 443!")
        (@arg CERT: -c --cert +takes_value "Directory to SSL certificate PEM file. Required if using port 443!")
        (@arg ROOT: +required "Website root directory.")
    ).get_matches();

    let config = Config::new(args).unwrap_or_else(|err| {
        eprintln!("Error occured parsing arguments: {}", err);
        process::exit(1);
    });

    let root = Path::new(&config.root[..]);
    assert!(env::set_current_dir(&root).is_ok());

    let mut key_present = false;
    let mut key_string = String::new();
    if let Some(key) = config.key{
        key_present = true;
        key_string = String::from(key);
    };

    let mut cert_string = String::new();
    if key_present {
        if let Some(cert) = config.cert {
            cert_string = String::from(cert);
        };
    }

    let mut acceptor = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls()).unwrap();
    if key_present {
        acceptor.set_private_key_file(key_string, SslFiletype::PEM).unwrap();
        acceptor.set_certificate_chain_file(cert_string).unwrap();
        acceptor.check_private_key().unwrap();
    }
    let acceptor = Arc::new(acceptor.build());

    let listener = TcpListener::bind(format!("0.0.0.0:{}", &config.port[..])).unwrap();

    let pool = ThreadPool::new(10);
    for stream in listener.incoming() {
        match stream { 
            Ok(stream) => {
                if key_present {
                    let acceptor = acceptor.clone();
                    let stream = match acceptor.accept(stream) {
                        Ok(stream) => stream,
                        Err(_) => { 
                            log(String::from("Acceptor had trouble creating a stream. Falling back."));
                            continue;
                        }
                    };
                    pool.execute(|| {
                        handle_connection(stream);
                    });
                } else {
                    pool.execute(|| {
                        handle_connection(stream);
                    });
                }
            },
            Err(e) => { log(format!("Connection failed! {}", e)) }
        }
    }

    log("Server ended abruptly! Rebuilding thread pool.".to_string());
}

fn handle_connection<T>(mut stream: T) 
    where
        T: Read + Write
{
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(read) => read,
        Err(_) => { 
            log(String::from("Unable to read buffer to stream!"));
            return;
        }
    };

    log(String::from_utf8_lossy(&buffer).to_string());

    let buffer = String::from_utf8_lossy(&buffer);
    let buffer: Vec<&str> = buffer.split_whitespace().collect();
    let mut uri = buffer[1].to_string();

    if uri == "/" {
        uri = "/index.html".to_string();
    }
    log(String::from(&uri));

    let uri = uri.replace("%20", " ");

    let (status_line, mut contents) = match fs::read(&uri[1..]) {
        Ok(c) => ("HTTP/1.1 200 OK", c),
        Err(_) => match fs::read("404.html") {
            Ok(c) => ("HTTP/1.1 404 NOT FOUND", c),
            Err(_) => {
                log(String::from("Unable to read file 404.html!"));
                return;
            }
        }
    };

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n",
        status_line,
        contents.len(),
    );

    let mut response = response.as_bytes().to_vec();
    response.append(&mut contents);

    match stream.write_all(&response[..]) {
        Ok(write) => write,
        Err(_) => { 
            log(String::from("Unabled to write all bytes as response to stream!"));
            return;
        }
    }
    match stream.flush() {
        Ok(flush) => flush,
        Err(_) => { 
            log(String::from("Unable to flush stream after returning bytes in response!"));
            return;
        }
    }
}

fn get_time() -> String {
    let system_time = SystemTime::now();
    let datetime: DateTime<Utc> = system_time.into();
    format!("{}", datetime.format("%d/%m/%Y %T"))
}

fn log(message: String) {
    println!("{}: {}", get_time(), message);
}

