use std::io;
use std::collections::HashMap;
mod TCP_connections;
mod dbengine;
mod planner;
use crate::dbengine::engine::*;
// use crate::TCP_connections::server::*;
fn combine_bytes(high_byte: u8, low_byte: u8) -> u16 {
    // Combine the two bytes into a u16
    let combined = ((high_byte as u16) << 8) | (low_byte as u16);
    combined
}

fn main() {
   
}