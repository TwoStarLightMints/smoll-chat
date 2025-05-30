use crate::error;
use std::convert::AsRef;
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

type SmollParseResult = Result<SmollChatOpts, error::OptionParsingError>;

pub struct SmollChatOpts {
    pub port: u32,
    pub qrcode: bool,
    pub static_dir: PathBuf,
    pub room_name: String,
    pub max_clients: usize,
}

impl SmollChatOpts {
    pub fn default() -> Self {
        Self {
            port: 8080,
            qrcode: false,
            static_dir: env::current_dir().unwrap(),
            room_name: String::from("Room"),
            max_clients: 10,
        }
    }

    pub fn parse() -> SmollParseResult {
        let env_file = File::open(".env");

        match env_file {
            Ok(file) => Ok(Self::parse_env(file)?),
            Err(_) => Self::parse_args(env::args()),
        }
    }

    fn parse_iter<T: Iterator<Item = impl AsRef<str>>>(mut iter: T) -> SmollParseResult {
        let mut opts_parsed = Self::default();

        while let Some(opt) = iter.next() {
            match opt.as_ref() {
                "--port" | "-p" => {
                    opts_parsed.port = match iter.next() {
                        Some(a) => match a.as_ref().parse::<u32>() {
                            Ok(port) => port,
                            Err(_) => {
                                return Err(error::OptionParsingError::InvalidValue(
                                    a.as_ref().to_string(),
                                ))
                            }
                        },
                        None => {
                            return Err(error::OptionParsingError::NoValueFound);
                        }
                    };
                }
                "--qrcode" => {
                    opts_parsed.qrcode = match iter.next() {
                        Some(v) => match v.as_ref().parse::<bool>() {
                            Ok(value) => value,
                            Err(_) => {
                                return Err(error::OptionParsingError::InvalidValue(
                                    v.as_ref().to_string(),
                                ))
                            }
                        },
                        None => {
                            return Err(error::OptionParsingError::NoValueFound);
                        }
                    }
                }
                "--static-dir" => {
                    opts_parsed.static_dir = match iter.next() {
                        Some(static_dir) => PathBuf::from(static_dir.as_ref()),
                        None => return Err(error::OptionParsingError::NoValueFound),
                    }
                }
                "--room-name" => {
                    opts_parsed.room_name = match iter.next() {
                        Some(v) => v.as_ref().to_string(),
                        None => return Err(error::OptionParsingError::NoValueFound),
                    }
                }
                "--max-clients" => {
                    opts_parsed.max_clients = match iter.next() {
                        Some(v) => match v.as_ref().parse::<usize>() {
                            Ok(max_clients) => max_clients,
                            Err(_) => {
                                return Err(error::OptionParsingError::InvalidValue(
                                    v.as_ref().to_string(),
                                ));
                            }
                        },
                        None => return Err(error::OptionParsingError::NoValueFound),
                    }
                }
                _ => {
                    return Err(error::OptionParsingError::InvalidKey(
                        opt.as_ref().to_string(),
                    ))
                }
            }
        }

        Ok(opts_parsed)
    }

    pub fn parse_args(args: env::Args) -> SmollParseResult {
        Self::parse_iter(args)
    }

    pub fn parse_env(mut env_file: File) -> SmollParseResult {
        let mut opts_parsed = Self::default();

        let mut buf = String::new();

        env_file
            .read_to_string(&mut buf)
            .expect("Error reading opts file");

        let mut opts = buf.split_whitespace();

        while let Some(opt) = opts.next() {
            let mut pair = opt.split("=");
            let key = pair.next().unwrap();
            let value = pair.next();

            match key {
                "port" => {
                    opts_parsed.port = match value {
                        Some(v) => match v.parse::<u32>() {
                            Ok(port) => port,
                            Err(_) => {
                                return Err(error::OptionParsingError::InvalidValue(v.to_string()));
                            }
                        },
                        None => {
                            return Err(error::OptionParsingError::NoValueFound);
                        }
                    }
                }
                "qrcode" => {
                    opts_parsed.qrcode = match value {
                        Some(v) => match v.parse::<bool>() {
                            Ok(boolean) => boolean,
                            Err(_) => {
                                return Err(error::OptionParsingError::InvalidValue(v.to_string()))
                            }
                        },
                        None => return Err(error::OptionParsingError::NoValueFound),
                    }
                }
                "static-dir" => {
                    opts_parsed.static_dir = match value {
                        Some(v) => PathBuf::from(v),
                        None => return Err(error::OptionParsingError::NoValueFound),
                    }
                }
                "room-name" => {
                    opts_parsed.room_name = match value {
                        Some(v) => v.to_string(),
                        None => return Err(error::OptionParsingError::NoValueFound),
                    }
                }
                "max-clients" => {
                    opts_parsed.max_clients = match value {
                        Some(v) => match v.parse::<usize>() {
                            Ok(max_clients) => max_clients,
                            Err(_) => {
                                return Err(error::OptionParsingError::InvalidValue(v.to_string()));
                            }
                        },
                        None => return Err(error::OptionParsingError::NoValueFound),
                    }
                }
                _ => return Err(error::OptionParsingError::InvalidKey(key.to_string())),
            }
        }

        return Ok(opts_parsed);
    }
}

impl Display for SmollChatOpts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Server Config\nPort Number: {}\nQR Code Enabled: {}\nStatic Directory: {}\nRoom Name: {}\nMax Number of Clients: {}",
            self.port,
            if self.qrcode {"Yes"} else {"No"},
            self.static_dir.to_str().unwrap(),
            self.room_name,
            self.max_clients
        )
    }
}
