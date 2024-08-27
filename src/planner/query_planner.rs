use crate::dbengine::btrees::*;
use crate::dbengine::pages::*;
use crate::dbengine::buffer_manager::*;


pub enum Operation {
    Insert(),
    Delete(),
    Update(),
    Read(),
    
}
pub struct QueryExecutor {
    operation: Operation,
    bplustree: BPlusTree,
}