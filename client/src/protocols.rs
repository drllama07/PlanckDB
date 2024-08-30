
use std::fmt;

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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(_, s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyRow {
    pub key: Value,
    pub row: Vec<Value>
}

impl fmt::Display for KeyRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "-> {} | [{}]", self.key, self.row.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(", "))
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub column_number: u8,
    pub column_names: Vec<String>,
    pub column_types: Vec<u8>,
    pub page_id_count: u32,
    pub root_node_offset: u32,
    pub free_page_num: u32,
    pub free_page_list: Vec<u32>
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Table: {}", self.name)?;
        writeln!(f, "Columns:")?;
        for (name, col_type) in self.column_names.iter().zip(self.column_types.iter()) {
            write!(f, "  - {} (type: {})", name, col_type)?;
        }
        Ok(())
    }
}

pub struct Packet {
    pub table: Table,
    pub keyrows: Vec<KeyRow>
}


impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.table)?;
        writeln!(f, "Keys and Rows:")?;
        for (i, keyrow) in self.keyrows.iter().enumerate() {
            write!(f, "  {}: {}", i + 1, keyrow)?;
        }
        Ok(())
    }
}

pub enum Response {
    Query(String),
    Return(Packet),
    Error(String),
}

fn serialize_table(table: Table) -> Vec<u8> {
    let mut vec:Vec<u8> = Vec::new();
    vec.extend_from_slice(&[0;2]);

    vec.push(table.name.len() as u8);
    vec.extend_from_slice(table.name.as_bytes());
    vec.push(table.column_types.len() as u8);
    for (i,types) in table.column_types.into_iter().enumerate() {
        vec.push(types);
        vec.push(table.column_names[i].len() as u8);
        vec.extend_from_slice(table.column_names[i].as_bytes())
    };
    let size = vec.len() as u16;
    vec[0..2].copy_from_slice(&(size - 2).to_be_bytes());
    return vec;
}

fn deconstruct_table(data: &[u8]) -> Table {
   let len_tmp = data[0] as usize ;
   let mut pointer = 1;
   let name: String = std::str::from_utf8(&data[pointer..pointer+len_tmp]).unwrap().to_string();
   pointer += len_tmp;
   let len_tmp = data[pointer];
   pointer += 1;
   let mut column_types = Vec::new();
   let mut column_names = Vec::new();
   for i in 0..len_tmp {
       column_types.push(data[pointer]);
       pointer += 1;
       let tmp_len = data[pointer] as usize;
       pointer += 1;
       column_names.push(std::str::from_utf8(&data[pointer..pointer + tmp_len]).unwrap().to_string());
       pointer += tmp_len;

   }
   // Most of the values are empty or zero because we do not really need them from now on.
   return Table { name: name, column_number: 0, column_names: column_names, column_types: column_types, page_id_count: 0, root_node_offset: 0, free_page_num: 0, free_page_list: vec![] };
}

fn serialize_keyrow(mut kr: Vec<KeyRow>) -> Vec<u8> {
    let mut vec: Vec<u8> = Vec::new();
    vec.extend_from_slice(&[0;2]);
    vec.push(kr.len() as u8);
    let first = kr.remove(0);
    match first.key {
        Value::Number(x) => {
            vec.push(2);
            vec.extend_from_slice(&x.to_be_bytes());
        }
        Value::String(len, string) => {
            vec.push(7);
            vec.push(len);
            vec.extend_from_slice(string.as_bytes());
        }
    }
    vec.push(first.row.len() as u8);
    for row in first.row {
        match row {    
            Value::Number(x) => {
                vec.extend_from_slice(&x.to_be_bytes());
            }
            Value::String(len, string) => {
                vec.push(len);
                vec.extend_from_slice(string.as_bytes());
            }
        }
    }
    for keyrow in kr {
        match keyrow.key {
            Value::Number(x) => {
                vec.extend_from_slice(&x.to_be_bytes());
            }
            Value::String(len, string) => {
                vec.push(len);
                vec.extend_from_slice(string.as_bytes());
            }
        };

        vec.push(keyrow.row.len() as u8);
        for row in keyrow.row {
            match row {    
                Value::Number(x) => {
                    vec.extend_from_slice(&x.to_be_bytes());
                }
                Value::String(len, string) => {
                    vec.push(len);
                    vec.extend_from_slice(string.as_bytes());
                }
            }
        }

    }
    let size = vec.len() as u16;
    vec[0..2].copy_from_slice(&(size - 2).to_be_bytes());
    return vec;
}


fn deconstruct_keyrow(data: &[u8], column_types: &Vec<u8>) -> Vec<KeyRow> {
    
    let mut vec = Vec::new();
    let mut pointer = 0;
    let tmp_len = data[pointer];
    pointer += 1;
    let is_string = match data[pointer]{
        2 => false ,
        7 => true,
        _ => panic!()
    };
    pointer += 1;
    for i in 0..tmp_len {
        if is_string {
           let tmp_len = data[pointer] as usize;
           pointer += 1;
           let string = std::str::from_utf8(&data[pointer..pointer+tmp_len]).unwrap().to_string();
           pointer += tmp_len;
           let mut row_vec = Vec::new();
           for types in column_types {
               match types {
                  2 => {
                    let vl = &data[pointer..pointer+4];
                    pointer += 4;
                    if vl.len() == 4 {
                        let array: [u8; 4] = [vl[0], vl[1], vl[2], vl[3]];
                        row_vec.push(Value::Number(u32::from_be_bytes(array)));
                    } else {
                        panic!()
                    }
                  },
                  7 => {
                        let tmp_len = data[pointer] as usize;
                        pointer += 1;
                        let str = std::str::from_utf8(&data[pointer..pointer+tmp_len]).unwrap().to_string();
                        pointer += tmp_len;
                        row_vec.push(Value::string(str))
                  }
                  _ => panic!()
               }
           }
           vec.push(KeyRow{key: Value::string(string), row: row_vec})
        } else {
            let vl = &data[pointer..pointer+4];
            pointer += 4;
            let tmp_len = data[pointer];
            if !(tmp_len as usize == column_types.len()) {
                panic!()
            }
            pointer += 1;
            let mut key;
            if vl.len() == 4 {
                let array: [u8; 4] = [vl[0], vl[1], vl[2], vl[3]];
                key = Value::Number(u32::from_be_bytes(array));
            } else {
                panic!()
            } 
            let mut row_vec = Vec::new();
            for types in column_types {
                match types {
                    2 => {
                        let vl = &data[pointer..pointer+4];
                        pointer += 4;
                        if vl.len() == 4 {
                            let array: [u8; 4] = [vl[0], vl[1], vl[2], vl[3]];
                            row_vec.push(Value::Number(u32::from_be_bytes(array)));
                        } else {
                            panic!()
                        }
                    },
                    7 => {
                            let tmp_len = data[pointer] as usize;
                            pointer += 1;
                            let str = std::str::from_utf8(&data[pointer..pointer+tmp_len]).unwrap().to_string();
                            pointer += tmp_len;
                            row_vec.push(Value::string(str))
                    }
                    x => panic!("{}", x)
                }
            }
            vec.push(KeyRow{key: key, row: row_vec})
        }
    }
    return vec;
}

pub fn serialize(response: Response) -> Vec<u8> {
    let mut packet = Vec::from(0u32.to_be_bytes());
    
    match response {
        Response::Query(s) => {
            packet.push(b'+');
            let len = s.len() as u16;
            packet.extend_from_slice(&len.to_be_bytes());
            packet.extend_from_slice(s.as_bytes());
            let size = packet.len() as u32 - 4;
            packet[0..4].copy_from_slice(&size.to_be_bytes());
        },
        Response::Return(pack) => {
              packet.push(b'=');
              packet.extend(serialize_table(pack.table));
              packet.extend(serialize_keyrow(pack.keyrows));
              let size = packet.len() as u32 - 4;
              packet[0..4].copy_from_slice(&size.to_be_bytes());
        },
        Response::Error(s) => {
            packet.push(b'!');
            let len = s.len() as u16;
            packet.extend_from_slice(&len.to_be_bytes());
            packet.extend_from_slice(s.as_bytes());
            let size = packet.len() as u32 - 4;
            packet[0..4].copy_from_slice(&size.to_be_bytes());
        }
    }
    return packet;
}


pub fn deserialize(response: &[u8]) -> Response {
    let mut pointer = 1;
    match response[0] {
        b'+' => {
            let vl = &response[pointer..pointer +2];
            pointer += 2;
            let mut number;
            if vl.len() == 2 {
                number = [vl[0], vl[1]];
            }
            else {
                panic!()
            }
            let mut len_tmp = u16::from_be_bytes(number) as usize;
             let s = std::str::from_utf8(&response[pointer..pointer + len_tmp]).unwrap().to_string();
             pointer += len_tmp;
             return Response::Query(s) 
        },
        b'=' => {
             let vl = &response[pointer..pointer +2];
             pointer += 2;
             let mut number;
             if vl.len() == 2 {
                 number = [vl[0], vl[1]];
             }
             else {
                 panic!()
             }
             let mut len_tmp = u16::from_be_bytes(number) as usize;
             let table = deconstruct_table(&response[pointer..pointer+ len_tmp]);
             pointer += len_tmp;
             let vl = &response[pointer..pointer +2];
             pointer += 2;
             let mut number;
             if vl.len() == 2 {
                 number = [vl[0], vl[1]];
             }
             else {
                 panic!()
             }
             let mut len_tmp = u16::from_be_bytes(number) as usize;
             let keyrow = deconstruct_keyrow(&response[pointer..pointer+len_tmp], &table.column_types);

             Response::Return(Packet { table: table, keyrows: keyrow })
        },
        b'!' => {
            let vl = &response[pointer..pointer +2];
            pointer += 2;
            let mut number;
            if vl.len() == 2 {
                number = [vl[0], vl[1]];
            }
            else {
                panic!()
            }
            let mut len_tmp = u16::from_be_bytes(number) as usize;
             let s = std::str::from_utf8(&response[pointer..pointer + len_tmp]).unwrap().to_string();
             pointer += len_tmp;
             return Response::Error(s) 
        },
        _ => {
            panic!()
        },

    }
    
}
