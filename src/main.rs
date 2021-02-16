mod ftpc;
extern crate port_scanner;
extern crate text_io;
use std::env;
use std::io::{Error, ErrorKind, Write, Result};
use text_io::read;

fn run(ftpconnection: &mut ftpc::FTPConnection) -> Result<()> {
    loop {
        print!("ftpc> ");
        std::io::stdout().flush()?;
        let input: String = read!("{}\n");
        let input_splitted: Vec<&str> = input.split(" ").collect();
        if input_splitted[0] == "login" && input_splitted.len() >= 3 {
            ftpconnection.handle_command(ftpc::Command::User(
                ftpc::USER_BASE_COMMAND,
                String::from(input_splitted[1]),
            ))?;
            ftpconnection.handle_command(ftpc::Command::Pass(
                ftpc::PASS_BASE_COMMAND,
                String::from(input_splitted[2]),
            ))?;
        } else if input_splitted[0] == "dir" || input_splitted[0] == "ls" {
        	let mut directory: String = "".to_string();
        	if input_splitted.len() == 2{
        		directory = String::from(input_splitted[1]);
        	}
            let random_port = port_scanner::request_open_port().ok_or(Error::new(ErrorKind::Other, "Failed getting an open port"))?;
            ftpconnection.handle_command(ftpc::Command::Port(ftpc::PORT_BASE_COMMAND, random_port))?;
            ftpconnection.handle_command(ftpc::Command::List(ftpc::LIST_BASE_COMMAND, directory, random_port))?;
        } else if input_splitted[0] == "cd" && input_splitted.len() >= 2 {
            ftpconnection.handle_command(ftpc::Command::Cwd(
                ftpc::CWD_BASE_COMMAND,
                String::from(input_splitted[1]),
            ))?;
        } else if input_splitted[0] == "get" && input_splitted.len() == 3 {
        	let pathname = String::from(input_splitted[1]);
        	let local_path = String::from(input_splitted[2]);
        	let random_port = port_scanner::request_open_port().ok_or(Error::new(ErrorKind::Other, "Failed getting an open port"))?;
        	ftpconnection.handle_command(ftpc::Command::Port(ftpc::PORT_BASE_COMMAND, random_port))?;
        	ftpconnection.handle_command(ftpc::Command::Retr(ftpc::RETR_BASE_COMMAND, pathname, local_path, random_port))?;
        } else if input_splitted[0] == "quit" || input_splitted[0] == "bye" {
            ftpconnection.handle_command(ftpc::Command::Quit(ftpc::QUIT_BASE_COMMAND))?;
            break;
        }
        else {
        	println!("Invalid command");
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(Error::new(ErrorKind::Other, "Not Enough arguments"));
    }
    let mut ftpconnection = ftpc::FTPConnection::new(args[1].as_str())?;
    run(&mut ftpconnection)?;
    Ok(())
}
