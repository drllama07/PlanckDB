use std::{
    net,
    io::Error,
    io::prelude::*,
    net::{TcpListener, TcpStream},
};



pub fn server() -> String {
    
    // this ensures that two cennections dont interrupt each other while changing tasks
    println!("Started to listen at 127.0.0.1:7878 <-");
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let mut commands: String;
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        // Buffer to hold the received data (16 bytes for two usize values)
        let mut buffer = [0; 64];
        stream.read(&mut buffer).unwrap();

        // Convert bytes to usize values
        commands = String::from_utf8(buffer[0..64].try_into().unwrap()).unwrap();
        
        return commands;
        
    }
    return "".to_string();
}