use crate::dbengine::btrees::*;
use std::{collections::HashMap, fmt::Error};


fn combine_bytes(high_byte: u8, low_byte: u8) -> u16 {
    // Combine the two bytes into a u16
    let combined = ((high_byte as u16) << 8) | (low_byte as u16);
    combined
}

fn bytes_to_u32(buffer: &[u8], offset: usize) -> u32 {
    // Ensure that the slice contains exactly 4 bytes to safely convert to u32
    let bytes = &buffer[offset..offset + 4];
    
    // Convert using big-endian order (change to from_le_bytes for little-endian)
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn row_size(row: &Vec<Value>) -> u8 {
    let mut size = 0;
    for value in row {
        match  value {
            Value::Number(x) => size += 4,
            Value::String(len, x) => size += len + 1,
        }
    };
    return size;
    

}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Number(u32),
    String(u8, String)
}

impl Value {
    pub fn string(str: String) -> Self {
        let bytes = str.as_bytes();
        Value::String(bytes.len() as u8, str)
    }

    pub fn extract_pointer(&self) -> u32 {
        match  self {
            Value::Number(x) => *x,
            _ => panic!("No pointer")
        }
    }
}
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Slot {
    pub value: Value,
    pub pointer: u16,
}
#[derive(Debug, Clone)]
pub struct Rows {
    pub size: u8,
    pub values: Vec<Value>
}

#[derive(Debug, Clone)]
pub struct Page {
    pub page_type: NodeType,
    pub free_space_pointer: u16,
    pub slots: Vec<Slot>,
    pub cells: HashMap<u16,Rows>
}
#[derive(Debug, Clone)]
pub struct KeyRow {
    pub key: Value,
    pub row: Vec<Value>
}

impl Page {
    pub fn new_leaf() -> Self {
        Page { page_type: NodeType::Leaf(Vec::new()), free_space_pointer: 4093, slots: Vec::new(), cells: HashMap::new() }
    }
    pub fn new_internal() -> Self {
        Page { page_type: NodeType::Internal(Vec::new()), free_space_pointer: 4093, slots: Vec::new(), cells: HashMap::new() }
    }

    pub fn clean_page(&mut self) {
        self.slots = Vec::new();
        self.cells = HashMap::new();
        self.free_space_pointer = 4095;
    }

    pub fn mut_slot(&mut self, key: Value) -> &mut Slot {
        let mut idx = 0;
        for slot in &self.slots {
            if slot.value == key {
                break;
            }
            idx += 1;
        }

        &mut self.slots[idx]
    }
   
    pub fn insert(&mut self,kv: KeyRow) {
          let row_size = row_size(&kv.row);
          
          let mut index = 0;
          for slot in &self.slots {
            if slot.value > kv.key {
                break;
            }
            index += 1;
          }
          let pointer = self.free_space_pointer - row_size as u16 - 1;
          self.slots.insert(index, Slot { value: kv.key, pointer:  pointer });
          self.cells.insert(pointer, Rows { size: row_size, values: kv.row });
          self.free_space_pointer = pointer;
          
    }


    pub fn delete(&mut self, key: Value){
        let mut index = 0;
        for slot in &self.slots {
            if slot.value == key {
                let removed = self.slots.remove(index);            
                self.cells.remove(&removed.pointer);
                break;
            }
            index += 1;
        }  
    }

    pub fn page_to_buff(&self) -> Result<[u8; 4096], Error>{
        let mut buffer: [u8; 4096] = [0; 4096];
        let mut offset = 0;
        let mut free_space_pointer: u16 = 4095;
        match &self.page_type {
            NodeType::Leaf(_) => {
                buffer[0] = 1;
            },
            NodeType::Internal(_) => {
                buffer[0] = 0;
            }
        };
        offset += 1;

        // buffer[offset..offset + 2].copy_from_slice(&self.free_space_pointer.to_be_bytes());
        
        offset += 3;
        
        buffer[offset] = self.slots.len() as u8;
        offset += 1;

        for slot in &self.slots {
            match &slot.value {
                Value::Number(x) => {
                    buffer[3] = 0;
                    buffer[offset..offset + 4].copy_from_slice(&x.to_be_bytes());
                    offset += 4;
                    buffer[offset..offset + 2].copy_from_slice(&slot.pointer.to_be_bytes());
                    if free_space_pointer > slot.pointer {
                        free_space_pointer = slot.pointer;
                    }
                    offset += 2;
                    
                    let row = &self.cells.get(&slot.pointer).unwrap();
                    let mut internal_offset = slot.pointer as usize;
                    if internal_offset <= offset {
                        return Err(Error);
                    }
                    
                    buffer[internal_offset]= row.size;
                    internal_offset += 1;
                    for value in &row.values {
                        match value {
                            Value::Number(xi) => {
                                buffer[internal_offset..internal_offset + 4].copy_from_slice(&xi.to_be_bytes());
                                internal_offset += 4;
                            },
                            Value::String(size, xi) => {
                                buffer[internal_offset] = *size;
                                internal_offset += 1;
                                buffer[internal_offset..internal_offset + *size as usize].copy_from_slice(&xi.as_bytes());
                                internal_offset += *size as usize;
                            }
                        }
                    }
                    
                    
                },
                Value::String(size, x) => {
                    buffer[3] = 1;
                    buffer[offset] = *size;
                    offset += 1;
                    buffer[offset..offset + *size as usize].copy_from_slice(&x.as_bytes());
                    offset += *size as usize;
                    buffer[offset..offset + 2].copy_from_slice(&slot.pointer.to_be_bytes());
                    if free_space_pointer > slot.pointer {
                        free_space_pointer = slot.pointer;
                    }
                    offset += 2;

                    
                    let row = &self.cells.get(&slot.pointer).unwrap();
                    let mut internal_offset = slot.pointer as usize;
                    if internal_offset <= offset {
                        return Err(Error);
                    }
                    buffer[internal_offset]= row.size;
                    internal_offset += 1;
                    for value in &row.values {
                        match value {
                            Value::Number(xi) => {
                                buffer[internal_offset..internal_offset + 4].copy_from_slice(&xi.to_be_bytes());
                                internal_offset += 4;
                            },
                            Value::String(size, xi) => {
                                buffer[internal_offset] = *size;
                                internal_offset += 1;
                                buffer[internal_offset..internal_offset + *size as usize].copy_from_slice(&xi.as_bytes());
                                internal_offset += *size as usize;
                            }
                        }
                    }
                    
                }
            }
        }
        buffer[1..3].copy_from_slice(&free_space_pointer.to_be_bytes());

        return Ok(buffer)
    }

    pub fn buff_to_page(field_types: Vec<u8>,buffer: [u8; 4096]) -> Page {
        let mut offset = 0;
        let mut free_space_pointer: u16 = 4095;

        let mut node_type ;

        if buffer[offset] == 1 {
            node_type = NodeType::Leaf(Vec::new());
        } else {
            node_type = NodeType::Internal(Vec::new());
        }

        offset += 1;
         
        free_space_pointer = combine_bytes(buffer[offset], buffer[offset+1]);
        offset += 2;

        let is_string = buffer[offset] == 1;
        offset += 1;

        let slot_count = buffer[offset];
        offset += 1;
        let mut slot_vec = Vec::new();
        if is_string {
            for i in 0..slot_count {
               let len: u8 = buffer[offset];
               offset += 1;
               
               let string = std::str::from_utf8(&buffer[offset..offset+len as usize]).unwrap().to_string();
               offset += len as usize;
               let pointer = combine_bytes(buffer[offset], buffer[offset+1]);
               offset += 2;
               slot_vec.push(Slot { value: Value::String(len, string), pointer: pointer })

            }
        } else {
          for i in 0..slot_count {
             let number = bytes_to_u32(&buffer, offset);
             offset += 4;
             let pointer = combine_bytes(buffer[offset], buffer[offset+1]);
             offset += 2;
             slot_vec.push(Slot{value: Value::Number(number), pointer: pointer})
          }
        }
        let mut cells = HashMap::new();
        let mut int_pointer = 0;
        match node_type {
            NodeType::Internal(_) => {
                for slot in &slot_vec {
                    let mut int_pointer = slot.pointer;
                    let size = buffer[int_pointer as usize];
                    int_pointer += 1;
                    let slice = &buffer[int_pointer as usize..(int_pointer + size as u16) as usize];
                    let mut row = Rows { size: size, values: Vec::new() };
                    
                    let mut slice_pointer = 0;
                    row.values.push(Value::Number(u32::from_be_bytes(slice[slice_pointer as usize..(slice_pointer+4) as usize].try_into().unwrap())));
                    slice_pointer += 4;
                    cells.insert(slot.pointer, row);
                }
            }
            NodeType::Leaf(_) => {
                for slot in &slot_vec {
                    let mut int_pointer = slot.pointer;
                    let size = buffer[int_pointer as usize];
                    int_pointer += 1;
                    let slice = &buffer[int_pointer as usize..(int_pointer + size as u16) as usize];
                    let mut row = Rows { size: size, values: Vec::new() };
                    
                    let mut slice_pointer = 0;
                    for field_type in &field_types {
                        if field_type == &2 {
                           row.values.push(Value::Number(u32::from_be_bytes(slice[slice_pointer as usize..(slice_pointer+4) as usize].try_into().unwrap())));
                           slice_pointer += 4;
                        } else if field_type == &7 {
                           let str_len =  slice[slice_pointer as usize];
                           slice_pointer += 1;
                           let string = std::str::from_utf8(&slice[slice_pointer as usize..(slice_pointer + str_len as u16) as usize]).unwrap();
                           row.values.push(Value::String(str_len, string.to_string()));
                           slice_pointer += str_len as u16;
                        }
                    }
                    cells.insert(slot.pointer, row);
                }
            }
        }
        
        
        if free_space_pointer == 4093 {
        
        }

        return Page { page_type: node_type, free_space_pointer: free_space_pointer, slots: slot_vec, cells: cells}
    }
}




