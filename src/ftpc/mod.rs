mod utils;
use std::fs;
use std::io::{Error, ErrorKind, Read, Result, Write};
use std::net::{TcpListener, TcpStream};

// command name and an array of successful response codes
type BaseCommand = (&'static str, [&'static str; MAX_SUCCESS_RESPONSE_CODES]);

const MAX_SUCCESS_RESPONSE_CODES: usize = 5;
pub const USER_BASE_COMMAND: BaseCommand = ("USER", ["230", "331", "332", "000", "000"]);
pub const PASS_BASE_COMMAND: BaseCommand = ("PASS", ["230", "202", "332", "000", "000"]);
pub const PORT_BASE_COMMAND: BaseCommand = ("PORT", ["200", "000", "000", "000", "000"]);
pub const LIST_BASE_COMMAND: BaseCommand = ("LIST", ["125", "150", "226", "250", "000"]);
pub const CWD_BASE_COMMAND: BaseCommand = ("CWD", ["250", "000", "000", "000", "000"]);
pub const RETR_BASE_COMMAND: BaseCommand = ("RETR", ["125","150","110","226","250"]);
pub const QUIT_BASE_COMMAND: BaseCommand = ("QUIT", ["221", "000", "000", "000", "000"]);

pub enum Command {
    User(BaseCommand, String),
    Pass(BaseCommand, String),
    Port(BaseCommand, u16),
    Cwd(BaseCommand, String),
    List(BaseCommand, String, u16),
    Retr(BaseCommand, String, String, u16),
    Quit(BaseCommand),
}

pub struct FTPConnection {
    ftpcommand_stream: TcpStream,
    buffer: [u8; 64],
}

impl FTPConnection {
    pub fn new(connection_string: &str) -> Result<Self> {
        let mut f = FTPConnection {
            ftpcommand_stream: TcpStream::connect(connection_string)?,
            buffer: [0; 64],
        };
        let initial_message = f.read()?;
        if !initial_message.contains("220") {
            return Err(Error::new(ErrorKind::Other, initial_message));
        }
        Ok(f)
    }

    fn get_local_address(self: &mut Self) -> Result<String> {
        utils::convert_local_address(self.ftpcommand_stream.local_addr()?.ip())
    }

    fn write_string(self: &mut Self, command: String) -> Result<()> {
        self.ftpcommand_stream.write(command.as_bytes())?;
        self.ftpcommand_stream.flush()?;
        Ok(())
    }

    fn read(self: &mut Self) -> Result<String> {
        self.ftpcommand_stream.read(&mut self.buffer)?;
        let response: String;
        match String::from_utf8((&mut self.buffer).to_vec()) {
            Ok(s) => response = s,
            Err(_) => return Err(Error::new(ErrorKind::Other, "Failed to read from stream")),
        };
        self.buffer = [0; 64];
        Ok(response)
    }

    fn check_response(self: &mut Self, command: BaseCommand) -> Result<bool> {
        let response = self.read()?;
        print!("{}", response);
        if command.1.iter().any(|&i| response.contains(i)) {
            return Ok(true);
        }
        Err(Error::new(ErrorKind::Other, response))
    }

    fn write_and_check_command(
        self: &mut Self,
        command_str: String,
        basecommand: BaseCommand,
    ) -> Result<()> {
        self.write_string(command_str)?;
        self.check_response(basecommand)?;
        Ok(())
    }

    fn handle_data_channel(
        self: &mut Self,
        port: u16,
    ) -> Result<std::thread::JoinHandle<Result<Vec<u8>>>> {
        let local_address: String = self.get_local_address()?.replace(",",".");
        let handle = std::thread::spawn(move || -> Result<Vec<u8>> {
            let ftpdata_listener: TcpListener = TcpListener::bind(format!("{}:{}",local_address[..local_address.len()-1].to_string(), port))?;
            let data: Vec<u8>;
            match ftpdata_listener.accept() {
                Ok((stream, _addr)) => data = utils::stream_handler(stream)?,
                Err(e) => return Err(Error::new(ErrorKind::Other, e)),
            };
            Ok(data)
        });
        Ok(handle)
    }

    pub fn handle_command(self: &mut Self, command: Command) -> Result<()> {
        match command {
            Command::User(basecommand, username) => {
                self.write_and_check_command(
                    format!("{} {}\r\n", basecommand.0, username),
                    basecommand,
                )?;
            }
            Command::Pass(basecommand, password) => {
                self.write_and_check_command(
                    format!("{} {}\r\n", basecommand.0, password),
                    basecommand,
                )?;
            }
            Command::Port(basecommand, port) => {
                let local_address = self.get_local_address()?;
                self.write_and_check_command(
                    format!(
                        "{} {}{},{}\r\n",
                        basecommand.0,
                        local_address,
                        (port >> 8) as u8,
                        port as u8
                    ),
                    basecommand,
                )?;
            }
            Command::Cwd(basecommand, directory) => {
                self.write_and_check_command(
                    format!("{} {}\r\n", basecommand.0, directory),
                    basecommand,
                )?;
            }
            Command::List(basecommand, directory, data_port) => {
                let basecommand_clone = (basecommand.0.clone(), basecommand.1);
                let handle = self.handle_data_channel(data_port)?;
                let mut command = format!("{}\r\n", basecommand.0);
   				if directory != "" {
   					command = format!("{} {}\r\n", basecommand.0,directory);
   				}
                self.write_and_check_command(command, basecommand)?;
                match handle.join() {
                    Ok(s) => {
                        println!("{}", String::from_utf8_lossy(&s?));
                        self.check_response(basecommand_clone)?;
                    }
                    Err(_) => {
                        return Err(Error::new(ErrorKind::Other, "Failed to join data thread"))
                    }
                };
            }
            Command::Retr(basecommand, pathname, local_path, data_port) => {
                let basecommand_clone = (basecommand.0.clone(), basecommand.1);
                let handle = self.handle_data_channel(data_port)?;
                self.write_and_check_command(format!("{} {}\r\n", basecommand.0, pathname), basecommand)?;
                match handle.join() {
                    Ok(s) => {
                        fs::write(local_path, &s?)?;
                        self.check_response(basecommand_clone)?;
                    }
                    Err(_) => {
                        return Err(Error::new(ErrorKind::Other, "Failed to join data thread"))
                    }
                };

            }
            Command::Quit(basecommand) => {
                self.write_and_check_command(format!("{}\r\n", basecommand.0), basecommand)?;
            }
        };
        Ok(())
    }
}