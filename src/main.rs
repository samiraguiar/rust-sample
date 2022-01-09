use log::{debug, error, info};
use serde::Deserialize;
use std::fmt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

mod logging;

const DEFAULT_ADDRESS: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "6142";

/// Represents a User.
#[derive(Deserialize)]
struct User {
    id: i32,
    name: String,
    username: String,
    email: String,
    address: Option<String>,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let none = String::from("<none>");
        let address = self.address.as_ref().unwrap_or(&none);
        write!(
            f,
            "(id={}, name={}, username={}, email={}, address={})",
            self.id, self.name, self.username, self.email, address
        )
    }
}

/// Read from a TCP stream into a buffer until there is no more data.
async fn process_connection(stream: TcpStream) -> Result<String, String> {
    let mut stream = BufReader::new(stream);
    let mut input = String::new();

    loop {
        match stream.read_line(&mut input).await {
            Ok(0) => {
                debug!("Reached the end of stream");
                break;
            }
            Ok(_) => {}
            Err(err) => {
                return Err(format!("Error while reading from stream! {}", err));
            }
        }
    }

    debug!("Done fetching the contents of the stream");
    Ok(input)
}

/// Try to interpret a string as a valid user.
async fn parse_input(input: &str) -> Result<Vec<User>, String> {
    let users: Vec<User> = serde_json::from_str(input).unwrap();
    Ok(users)
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cmdline = clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            clap::Arg::new("ADDRESS")
                .short('a')
                .long("address")
                .help("Address to listen to")
                .default_value(DEFAULT_ADDRESS)
                .takes_value(true),
        )
        .arg(
            clap::Arg::new("PORT")
                .short('p')
                .long("port")
                .help("Port for this socket connection")
                .default_value(DEFAULT_PORT)
                .takes_value(true),
        )
        .arg(
            clap::Arg::new("LOG_LEVEL")
                .short('l')
                .long("level")
                .help("Set the log level (messages are sent to stderr)")
                .default_value("info")
                .possible_value("off")
                .possible_value("error")
                .possible_value("warn")
                .possible_value("info")
                .possible_value("debug")
                .possible_value("trace"),
        )
        .get_matches();

    // configure the log level
    let log_level_arg = cmdline
        .value_of("LOG_LEVEL")
        .ok_or_else(|| "--level requires an argument".to_string())?;

    let log_level = log_level_arg
        .parse::<::log::Level>()
        .map_err(|e| format!("invalid log level: {}", e))?;

    logging::init_logger(env!("CARGO_PKG_NAME"), Some(log_level))
        .map_err(|err| format!("Error to set up loggers: {}", err))?;

    let listen_addr = cmdline.value_of("ADDRESS").unwrap();
    let listen_port = cmdline.value_of("PORT").unwrap();
    let addr_port = format!("{}:{}", listen_addr, listen_port);

    info!("Binding to {}", addr_port);
    let listener = TcpListener::bind(addr_port)
        .await
        .expect("Could not bind to a TCP socket");

    loop {
        let (stream, _addr) = listener
            .accept()
            .await
            .expect("Could not accept connection");

        // Spawn our handler to be run asynchronously.
        tokio::spawn(async {
            debug!("accepted connection");

            let content = match process_connection(stream).await {
                Ok(content) => content,
                Err(e) => {
                    error!("an error occurred; error = {:?}", e);
                    return;
                }
            };

            let users = match parse_input(&content).await {
                Ok(users) => users,
                Err(e) => {
                    error!("an error occurred; error = {:?}", e);
                    return;
                }
            };

            for u in users {
                println!("=> Found user\n    {}", u);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parsing_user() {
        let input = r#"
            [
                {
                    "id": 2,
                    "name": "Ervin Howell",
                    "username": "Antonette",
                    "email": "Shanna@melissa.tv"
                },
                {
                    "id": 3,
                    "name": "Clementine Bauch",
                    "username": "Samantha",
                    "email": "Nathan@yesenia.net",
                    "address": "123 Red Blue Lake"
                }
            ]
        "#;

        let parsed = parse_input(&input).await.unwrap();
        // a few assertions should suffice
        assert_eq!(parsed[0].id, 2);
        assert_eq!(parsed[1].address.as_ref().unwrap(), "123 Red Blue Lake");
    }
}
