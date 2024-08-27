use std::{
    net,
    io::Error,
    io::prelude::*,
    net::{TcpListener, TcpStream},
};

pub fn client(commands:String)  -> Result<(), Error>{
    let mut stream = TcpStream::connect("127.0.0.1:7878").expect("error while connecting to addrs");
    println!("Connected to port --> 127.0.0.1:7878 <--");
    
    
    // Convert usize values to bytes
    let command_bytes = commands.as_bytes();

    // Create a buffer to hold both values
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&command_bytes);
    // Send the buffer to the server
    stream.write_all(&buffer)?;
    stream.shutdown(net::Shutdown::Both).unwrap();

    
    Ok(())
}