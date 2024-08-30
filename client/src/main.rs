use std::{
    net,
    io,
    io::Error,
    io::prelude::*,
    net::{TcpListener, TcpStream},
};

pub mod protocols;

use crate::protocols::*;

pub fn client()  -> Result<(), Error>{
    let mut stream = TcpStream::connect("127.0.0.1:7878").expect("error while connecting to addrs");
    println!("Connected to port --> 127.0.0.1:7878 <--");
    
    
    loop {
        let mut input = String::new();
        print!("PlanckDB -> ");
        io::stdout().flush()?;  // Ensure the prompt is displayed immediately
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") {
            break;
        };

        let response = Response::Query(input.to_string());

        let bytes = serialize(response);
        stream.write_all(&bytes)?;
        let mut buffer = [0;4];
        stream.read_exact(&mut buffer)?;
        let mut vec: Vec<u8> = Vec::new();
        vec.resize(u32::from_be_bytes(buffer) as usize, 0);
        stream.read_exact(&mut vec)?;

        let received_response = deserialize(&vec);

        match received_response {
            Response::Error(x) => {
                  eprintln!("DB --> {}",x);
            },
            Response::Query(x) => {
                  println!("DB --> {}",x);
            }
            Response::Return(packet) => {
                println!(" | Received a Packet | ");
                println!("{}", packet);
            }
        }

    }

    
    
    

    
    Ok(())
}


fn main() {
    println!(" | Welcome to PlackDB demo client-server app for testing |");
    client().unwrap();
    
}
