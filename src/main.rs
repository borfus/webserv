use webserv::ThreadPool;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::{fs, env, process, str};
use std::path::Path;
use std::vec::Vec;

#[macro_use]
extern crate clap;
use clap::ArgMatches;

struct Config {
    root: String,
    port: String
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

        Ok(Config{root, port})
    }
}

fn main() {
    let args = clap_app!(webserv =>
        (version: "1.0")
        (author: "Peter Bell")
        (about: "Basic HTTP Server")
        (@arg PORT: -p --port +takes_value "Port that the server will bind to")
        (@arg ROOT: +required "Website root directory")
    ).get_matches();

    let config = Config::new(args).unwrap_or_else(|err| {
        eprintln!("Error occured parsing arguments: {}", err);
        process::exit(1);
    });

    let listener = TcpListener::bind(format!("{}{}", "127.0.0.1:", &config.port[..])).unwrap();

    let root = Path::new(&config.root[..]);
    assert!(env::set_current_dir(&root).is_ok());

    let pool = ThreadPool::new(5);
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let buffer = str::from_utf8(&buffer).unwrap().to_string();
    let buffer : Vec<&str> = buffer.split_whitespace().collect();
    let mut uri = buffer[1];

    if uri == "/" {
        uri = "/index.html";
    }

    let uri = uri.replace("%20", " ");

    let (status_line, mut contents) = match fs::read(&uri[1..]) {
        Ok(c) => ("HTTP/1.1 200 OK", c),
        Err(_) => ("HTTP/1.1 404 NOT FOUND", fs::read("404.html").unwrap())
    };

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n",
        status_line,
        contents.len(),
    );

    let mut response = response.as_bytes().to_vec();
    response.append(&mut contents);

    stream.write(&response[..]).unwrap();
    stream.flush().unwrap();
}

