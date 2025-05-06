use crate::error;
use std::convert::AsRef;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

type SmollParseResult = Result<SmollChatOpts, error::OptionParsingError>;

pub struct SmollChatOpts {
    pub port: u32,
    pub qrcode: bool,
    pub static_dir: PathBuf,
    pub room_name: String,
}

impl SmollChatOpts {
    pub fn default() -> Self {
        Self {
            port: 8080,
            qrcode: false,
            static_dir: env::current_dir().unwrap(),
            room_name: String::from("Room"),
        }
    }

    pub fn parse() -> SmollParseResult {
        let env_file = File::open(".env");

        match env_file {
            Ok(file) => Ok(Self::parse_env(file)),
            Err(_) => Self::parse_args(env::args()),
        }
    }

    fn parse_iter<T: Iterator<Item = impl AsRef<str>>>(mut iter: T) -> SmollParseResult {
        let mut opts_parsed = Self::default();

        while let Some(opt) = iter.next() {
            match opt.as_str() {
                "--port" | "-p" => {
                    opts_parsed.port = match iter.next() {
                        Some(a) => match a.parse::<u32>() {
                            Ok(port) => port,
                            Err(_) => return Err(error::OptionParsingError::InvalidValue(a)),
                        },
                        None => {
                            return Err(error::OptionParsingError::NoValueFound);
                        }
                    };
                }
                "--qrcode" => {
                    opts_parsed.qrcode = iter
                        .next()
                        .expect("Not enough arguments passed")
                        .parse::<bool>()
                        .expect("Invalid boolean passed")
                }
                "--static-dir" => {
                    opts_parsed.static_dir =
                        PathBuf::from(iter.next().expect("Not enough arguments passed"))
                }
                "--room-name" => {
                    opts_parsed.room_name = iter.next().expect("Not enough arguments passed")
                }
                _ => return Err(error::OptionParsingError::InvalidKey(opt)),
            }
        }

        Ok(opts_parsed)
    }

    pub fn parse_args(mut args: env::Args) -> SmollParseResult {
        Self::parse_iter(args)
    }

    pub fn parse_env(mut env_file: File) -> SmollParseResult {
        let mut opts_parsed = Self::default();

        let mut buf = String::new();

        env_file
            .read_to_string(&mut buf)
            .expect("Error reading opts file");

        Self::parse_iter(buf.split_whitespace())
    }
}
