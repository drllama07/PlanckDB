use std::fmt::Error;
use std::fs;
use crate::dbengine::btrees::*;
use crate::dbengine::pages::*;
use crate::dbengine::engine::*;
use crate::TCP_connections::server::Commands;


pub fn create(file: &str,pk_index: u8, field_names: Vec<String>, field_types: Vec<u8>) -> Result<() , Error> {
    let path = format!("{}{}{}", "PlanckDB/", file, ".db");
    if fs::exists(path).unwrap() {
        return Err(Error);
    }
    Table::new(file, pk_index,field_names, field_types);
    return Ok(());
}

pub fn insert(kr: KeyRow, btree: &mut BPlusTree) {
       btree.insert(kr);
       btree.buffer_pool.flush_all();
}

pub fn delete(key: Value, btree: &mut BPlusTree) {
       btree.delete(key);
       btree.buffer_pool.flush_all();
}

pub fn update(kr: KeyRow, btree: &mut BPlusTree) {
    btree.update(kr);
    btree.buffer_pool.flush_all();
}

pub fn read(key: Value, btree: &mut BPlusTree) -> Result<KeyRow,Error>{
    let (id, parent_id) = btree.search(&key);
    let node = btree.buffer_pool.get(id);
    for slot in &node.slots {
        if slot.value == key {
            let row = &node.cells.get(&slot.pointer).unwrap();
            return Ok(KeyRow{key: key, row: row.values.clone()});
        }
    };
    return Err(Error)
}



// TODO, I can add more advance ways to get data if I can find time.
