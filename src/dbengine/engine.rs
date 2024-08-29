use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom, IoSliceMut};
use std::fmt;
// use crate::dbengine::btrees::*;
use crate::dbengine::pages::Page;


#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub pk_column: u8,
    // Column number is actually the end of the column bytes so sorry for the naming I was confussed too.
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

impl Table {

    pub fn print(&self) {
         // Print the table header
         println!("{:<15} | {:<15} | {:<15} | {:<15}", "Column Number", "Column Name", "Column Type", "Page ID Count");
         println!("{:-<60}", ""); // Separator line
 
         // Print the column data
         for (index, column_name) in self.column_names.iter().enumerate() {
             let column_type = match self.column_types[index] {
                 7 => "String",
                 2 => "Number",
                 _ => "Unknown",
             };
 
             println!("{:<15} | {:<15} | {:<15} | {:<15}", index + 1, column_name, column_type, self.page_id_count);
         }
 
         // Print additional metadata
         println!("\nRoot Node Offset: {}", self.root_node_offset);
         println!("Free Page Num: {}", self.free_page_num);
         println!("Free Page List: {:?}", self.free_page_list);
         println!("Primary Key Column Index: {}", self.pk_column);
    }
    pub fn new(table_name: &str,pk_index: u8, field_names: Vec<String>, field_types: Vec<u8>) -> Self {
        let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)  // This will create the file if it doesn't exist
        .open(format!("{}{}{}", "PlanckDB/", table_name, ".db")).unwrap();
        
        // file.write_all(header.as_bytes()).unwrap();
        let root_node_offset: u32 = 0;
        let  free_page_num: u32 = 0;
        let free_page_list: Vec<u32> = Vec::new();
        let page_id_count: u32 = 0;
        let mut cursor = 32;
        file.seek(SeekFrom::Start(cursor)).unwrap(); // Assuming header is 32 bytes
        file.write_all(&[11 as u8]).unwrap();
        file.write_all(&[pk_index]).unwrap();
        file.write_all(&[field_types.len() as u8]).unwrap();
        cursor += 3;
        let mut end_size = 0;
        let mut i = 0;
        for field in &field_names {
            file.write_all(&[field_types[i]]).unwrap();
            let size = field.as_bytes().len() as u8;
            file.write_all(&[size]).unwrap();
            file.write_all(field.as_bytes()).unwrap();
            end_size += size + 2;
            i += 1;
        }
        cursor += end_size as u64;
        file.seek(SeekFrom::Start(32)).unwrap();
        file.write_all(&[end_size]).unwrap();
        file.seek(SeekFrom::Start(cursor)).unwrap(); // Assuming header is 32 bytes
        
        file.write_all(&page_id_count.to_be_bytes()).unwrap();
        file.write_all(&root_node_offset.to_be_bytes()).unwrap();
        file.write_all(&free_page_num.to_be_bytes()).unwrap();
        
        for page in &free_page_list {
            file.write_all(&page.to_be_bytes()).unwrap();
        }
        
        let mut table = Table {name: table_name.to_string(), pk_column: pk_index,column_number: end_size, column_names: field_names, column_types: field_types, page_id_count: page_id_count, root_node_offset: root_node_offset, free_page_num: free_page_num, free_page_list: free_page_list };
        table.create_page(Page::new_leaf().page_to_buff().unwrap());
        table
    }

    pub fn update_table(&self) {
        let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)  // This will create the file if it doesn't exist
        .open(format!("{}{}{}", "PlanckDB/", self.name, ".db")).unwrap();
        
        // file.write_all(header.as_bytes()).unwrap();
        let root_node_offset: u32 = self.root_node_offset;
        let  free_page_num: u32 = self.free_page_num;
        let free_page_list: Vec<u32> = self.free_page_list.clone();
        let page_id_count: u32 = self.page_id_count;
        let field_names = self.column_names.clone();
        let field_types = self.column_types.clone();
        let pk_index = self.pk_column;
        let mut cursor = 32;
        file.seek(SeekFrom::Start(cursor)).unwrap(); // Assuming header is 32 bytes
        file.write_all(&[11 as u8]).unwrap();
        file.write_all(&[pk_index]).unwrap();
        file.write_all(&[field_types.len() as u8]).unwrap();
        cursor += 3;
        let mut end_size = 0;
        let mut i = 0;
        for field in &field_names {
            file.write_all(&[field_types[i]]).unwrap();
            let size = field.as_bytes().len() as u8;
            file.write_all(&[size]).unwrap();
            file.write_all(field.as_bytes()).unwrap();
            end_size += size + 2;
            i += 1;
        }
        cursor += end_size as u64;
        file.seek(SeekFrom::Start(32)).unwrap();
        file.write_all(&[end_size]).unwrap();
        file.seek(SeekFrom::Start(cursor)).unwrap(); // Assuming header is 32 bytes

        file.write_all(&page_id_count.to_be_bytes()).unwrap();
        file.write_all(&root_node_offset.to_be_bytes()).unwrap();
        file.write_all(&free_page_num.to_be_bytes()).unwrap();

        for page in &free_page_list {
            file.write_all(&page.to_be_bytes()).unwrap();
        }
    }
    
    pub fn read_table(table_name: &str) -> Table {
        let mut file = OpenOptions::new()
        .read(true)
        .write(false) 
        .open(format!("{}{}{}", "PlanckDB/", table_name, ".db")).unwrap();

        let mut column_number: u8;
        let mut pk_index: u8 ;
        let mut column_names: Vec<String>= Vec::new();
        let mut column_types: Vec<u8>  = Vec::new();
        let mut page_id_count: u32;
        let mut root_node_offset: u32;
        let mut free_page_num: u32;
        let mut free_page_list: Vec<u32> = Vec::new();
        file.seek(SeekFrom::Start(32)).unwrap();
        let mut temp4: [u8; 4] = [0;4];
        let mut temp2: [u8; 2] = [0;2];
        let mut temp1: [u8; 1] = [0;1];
        file.read(&mut temp1).unwrap();
        column_number = temp1[0];
        file.read(&mut temp1).unwrap();
        pk_index = temp1[0];
        file.read(&mut temp1).unwrap();
        let len = temp1[0];
        for ix in 0..len {
            file.read(&mut temp1).unwrap();
            column_types.push(temp1[0]);
            file.read(&mut temp1).unwrap();
            let str_len = temp1[0];   
            let mut buff = vec![0; str_len as usize];
            file.read(&mut buff).unwrap();
            column_names.push(String::from_utf8(buff).unwrap()); 
        }
        file.read(&mut temp4).unwrap();
        page_id_count = u32::from_be_bytes(temp4);
        file.read(&mut temp4).unwrap();
        root_node_offset= u32::from_be_bytes(temp4);
        file.read(&mut temp4).unwrap();
        free_page_num = u32::from_be_bytes(temp4);
        for id in 0..free_page_num {
            file.read(&mut temp4).unwrap();
            free_page_list.push(u32::from_be_bytes(temp4));
        }

        Table {name: table_name.to_string(),pk_column: pk_index,column_number: column_number, column_names: column_names, column_types: column_types, page_id_count: page_id_count, root_node_offset: root_node_offset, free_page_num: free_page_num, free_page_list: free_page_list }
    }   


    pub fn create_page(&mut self, buffer: [u8;4096]) -> u32{

        let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)  // This will create the file if it doesn't exist
        .open(format!("{}{}{}", "PlanckDB/", self.name, ".db")).unwrap();
        let header_end = (47 + self.column_number as u32 + 100 * 4) as u64;
        let page_id_new = self.page_id_count;
        if self.free_page_list.len() != 0 {
            let free_page_id = self.free_page_list.pop().unwrap();
            file.seek(SeekFrom::Start(header_end + free_page_id as u64 * 4096)).unwrap();
            file.write_all(&buffer).unwrap();
            self.update_table();
            return  free_page_id;
        } else {
            file.seek(SeekFrom::Start(header_end + page_id_new as u64 * 4096)).unwrap();
            file.write_all(&buffer).unwrap(); 
            self.page_id_count += 1;
            self.update_table();
            return page_id_new;
        }
       
    }

    pub fn update_page(&self, buffer: [u8;4096], page_id: u32){
        
        let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)  // This will create the file if it doesn't exist
        .open(format!("{}{}{}", "PlanckDB/", self.name, ".db")).unwrap();
        let header_end = (47 + self.column_number as u32 + 100 * 4) as u64;
        
        file.seek(SeekFrom::Start(header_end + page_id as u64 * 4096)).unwrap();
        file.write_all(&buffer).unwrap();

    }

    pub fn read_page(&self, page_id: u32) -> [u8;4096] {
        let mut file = OpenOptions::new()
        .read(true)
        .write(false) // This will create the file if it doesn't exist
        .open(format!("{}{}{}", "PlanckDB/", self.name, ".db")).unwrap();
        let header_end = (47 + self.column_number as u32 + 100 * 4) as u64;
        let mut buffer: [u8;4096] = [0; 4096];
        file.seek(SeekFrom::Start(header_end + page_id as u64 * 4096)).unwrap();
        file.read(&mut buffer).unwrap();
        return buffer;
    }
    pub fn remove_page(&mut self, page_id: u32) {
        self.update_page([0;4096], page_id);
        self.free_page_num += 1;
        self.free_page_list.push(page_id);

        while self.free_page_list.len() >= 100 {
            self.free_page_list.remove(0);
            self.free_page_num -= 1;
        }

        self.update_table();
    }
}



