use std::io::{Read, Result};
use std::net::{TcpStream, IpAddr};

pub fn stream_handler(mut stream: TcpStream) -> Result<Vec<u8>> {
    let mut buffer = [0; 1024];
    let mut result: Vec<u8> = Vec::new();
    loop {
        let bytes: usize = stream.read(&mut buffer)?;
        if bytes == 0 {
            return Ok(result);
        }
        result.extend_from_slice(&mut buffer[..bytes]);
    }
}

pub fn convert_local_address(ipaddr: IpAddr) -> Result<String> {
    let ip_digits: Vec<u8>;
    let mut ip_string = "".to_string().to_owned();
    match ipaddr {
        IpAddr::V4(ipv4) => ip_digits = ipv4.octets().to_vec(),
        IpAddr::V6(ipv6) => ip_digits = ipv6.octets().to_vec(), 
    };
    for i in ip_digits {
        ip_string.push_str(&i.to_string().to_owned());
        ip_string.push_str(&",".to_string().to_owned());
    }
    Ok(ip_string)
}