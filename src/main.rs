use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    ops::RangeInclusive,
    path::PathBuf,
    process, thread,
};

use serde::{Deserialize, Serialize};

use spinach::{term, Spinach};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Drips")]
#[command(author = "Zane Schaffer <personal.zane@gmail.com>")]
#[command(version = "1.0")]
#[command(about = "Transfer and recieve files over the internet", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// listens on localhost for files
    Listen {
        /// Port to listen on
        #[arg(value_parser = port_in_range)]
        port: u16,
    },

    /// sends a file to a listener
    Send {
        /// IPv4 address to connect to
        addr: String,

        /// File to send
        file: PathBuf,
    },
}

const PORT_RANGE: RangeInclusive<usize> = 1..=65535;

/// port validation
fn port_in_range(s: &str) -> Result<u16, String> {
    let port: usize = s
        .parse()
        .map_err(|_| format!(" `{}` isn't a port number", s))?;
    if PORT_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(format!(
            "Port not in range {}-{}",
            PORT_RANGE.start(),
            PORT_RANGE.end()
        ))
    }
}

/// recieve data from sender and write into a file
fn handle_stream(stream: &mut TcpStream) -> color_eyre::Result<()> {
    let s = Spinach::new("connecting");
    let mut reader = io::BufReader::new(stream);

    let mut buf = Vec::new();

    reader.read_until(b'\n', &mut buf).expect("metadata");
    let md: Metadata = bincode::deserialize(&buf[..buf.len()]).unwrap();

    let mut f = std::fs::File::create(&md.name).expect("new file");
    let mut buf = Vec::<u8>::new();
    reader.read_to_end(&mut buf)?;
    s.text(format!("Downloading {}", &md.name));
    f.write_all(&buf).expect("write all data from buffer");

    s.succeed(format!("Finished downloading {}", &md.name));
    Ok(())
}

/// name and size of file
#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    name: String,
    size: usize,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    ctrlc::set_handler(|| {
        term::show_cursor();
        std::process::exit(0);
    })?;

    let cli = Cli::parse();

    match &cli.command {
        Commands::Listen { port } => {
            let s = Spinach::new(format!("Listening on localhost:{}...", port));
            let listener = match TcpListener::bind(format!("localhost:{}", port)) {
                Ok(stream) => stream,
                Err(_) => {
                    println!("Couldn't bind to localhost:{}", port);
                    term::show_cursor();
                    process::exit(1)
                }
            };

            for stream in listener.incoming() {
                s.succeed("Found connection");
                match handle_stream(&mut stream?) {
                    Ok(_) => break,
                    Err(e) => {
                        println!("Couldn't finish transfer : {}", e);
                        term::show_cursor();
                        process::exit(1)
                    }
                }
            }
        }
        Commands::Send { addr, file } => {
            let mut stream = match TcpStream::connect(addr) {
                Ok(stream) => stream,
                Err(_) => {
                    println!("Couldn't connect to {}", addr);
                    term::show_cursor();
                    process::exit(1)
                }
            };

            let s = Spinach::new("connecting");

            let mut md = Metadata {
                name: file
                    .to_str()
                    .expect("filename should be valid string")
                    .to_string(),
                size: 0,
            };

            let file = std::fs::File::open(file).expect("file should be readable");

            md.size = file.metadata().unwrap().len() as usize;

            let mut reader = BufReader::new(file);
            let mut buffer = Vec::new();

            s.text("sending file");
            let sender = thread::spawn(move || {
                reader.read_to_end(&mut buffer).unwrap();

                let md = bincode::serialize(&md).unwrap();

                if let Err(e) = stream.write_all(&md) {
                    println!("Error: {:?}", e);
                    term::show_cursor();
                    process::exit(1)
                }

                stream.write_all(&[b'\n']).unwrap();
                if let Err(e) = stream.write_all(&buffer) {
                    println!("Error: {:?}", e);
                    term::show_cursor();
                    process::exit(1)
                }
            });

            sender.join().expect("wait until thread finishes");
            s.succeed("sent file!");
        }
    }
    Ok(())
}
