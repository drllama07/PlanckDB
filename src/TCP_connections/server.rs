use std::{
    fs, io::{prelude::*, Error}, net::{self, TcpListener, TcpStream}, process::Command, vec
};

use crate::dbengine::btrees::*;
use crate::dbengine::engine::*;
use crate::dbengine::pages::*;
use crate::TCP_connections::execute::*;
use crate::TCP_connections::protocols::*;


pub enum Commands {
    Insert(KeyRow),
    Delete(Value),
    Update(KeyRow),
    Read(Value),
    Create(String,u8,  Vec<String>, Vec<u8>),
    StartTransaction(String),
    CloseTransaction(),
    SyntaxError(String)
}

fn identify_value(input: &str) -> Value {
    if let Ok(x) = input.parse::<u32>() {
        Value::Number(x)
    } else {
        Value::string(input.to_string())
    }
}

fn parse(str: String, column_types: Option<&Vec<u8>>) -> Commands {
    let mut str_vec: Vec<&str> = str.split_whitespace().collect(); 

    while str_vec.len() > 0 {
         let lexeme = str_vec.remove(0);

         match lexeme {
            "create" => {
                if str_vec.len() < 3 {
                    return Commands::SyntaxError("Syntax: create table_name key -> type_of_key columns | Age 2 | Website 7 | ".to_string());
                }
                let file = str_vec.remove(0);
                let key_token = str_vec.remove(0);
                if key_token != "key" {
                    return Commands::SyntaxError("Syntax: create table_name key -> type_of_key columns | Age 2 | Website 7 | ".to_string());
                 }
                 if str_vec.remove(0) != "->" {
                    return Commands::SyntaxError("Syntax: create table_name key -> type_of_key columns | Age 2 | Website 7 | ".to_string());
                 }
                 let pk = str_vec.remove(0).parse::<u8>().unwrap();
                 let columns = str_vec.remove(0);
                if columns != "columns" {
                    return Commands::SyntaxError("Syntax: create table_name key -> type_of_key columns | Age 2 | Website 7 | ".to_string());
                 }
                 let mut name_vec = Vec::new();
                 let mut type_vec = Vec::new();
                while str_vec.len() > 1 {
                    if str_vec.remove(0) != "|" {
                        return Commands::SyntaxError("Syntax: create table_name key -> type_of_key columns | Age 2 | Website 7 | ".to_string());
                    }
                    let lexeme = str_vec.remove(0);
                    name_vec.push(lexeme.to_string());
                    let types = str_vec.remove(0);
                    match types.parse::<u8>() {
                        Ok(x) => {
                            type_vec.push(x);
                        }
                        _ => return Commands::SyntaxError("Syntax: create table_name key -> type_of_key columns | Age 2 | Website 7 | ".to_string())
                    }
                }
                if str_vec.remove(0) != "|" {
                    return Commands::SyntaxError("Syntax: create table_name key -> type_of_key columns | Age 2 | Website 7 | ".to_string());
                }

                return Commands::Create(file.to_string(), pk, name_vec, type_vec);
            },
            "open" => {
                if str_vec.len() < 1 {
                    return Commands::SyntaxError("You need to specifiy the file/table name".to_string());
                }
                let file = str_vec.remove(0);
                return Commands::StartTransaction(file.to_string());
            },
            "close" => {
                return Commands::CloseTransaction();
            }
            "insert" => {
                if str_vec.len() < 6 {
                    return Commands::SyntaxError("Syntax: insert key -> 1 row -> blabalabala ".to_string());
                }
                let key_token = str_vec.remove(0);
                if key_token != "key" {
                   return Commands::SyntaxError("Syntax: insert key -> 1 row -> blabalabala ".to_string());
                }
                if str_vec.remove(0) != "->" {
                    return  Commands::SyntaxError("Syntax: insert key -> 1 row -> blabalabala ".to_string());
                }
                // TODO Youy can change it so that by pk_index you can know the type 
                let key_value = identify_value(str_vec.remove(0));

                if str_vec.remove(0) != "row" {
                    return Commands::SyntaxError("Syntax: insert key -> 1 row -> blabalabala ".to_string());
                }
                if str_vec.remove(0) != "->" {
                    return  Commands::SyntaxError("Syntax: insert key -> 1 row -> blabalabala ".to_string());
                }
                let mut row = Vec::new();
                for types in column_types.unwrap() {
                    match types {
                        2 => {
                            let lexeme = str_vec.remove(0);
                            let number = lexeme.parse::<u32>().unwrap();
                            row.push(Value::Number(number));
                        }

                        7 => {
                            let lexeme = str_vec.remove(0);
                            row.push(Value::string(lexeme.to_string()))
                        }
                        _ => panic!()
                    }
                }

                return  Commands::Insert(KeyRow { key: key_value, row: row }); 
            },
            "delete" => {
                if str_vec.len() < 3 {
                    return Commands::SyntaxError("Syntax: delete key -> 1 ".to_string());
                }
                let key_token = str_vec.remove(0);
                if key_token != "key" {
                    return Commands::SyntaxError("Syntax: delete key -> 1 ".to_string());
                }
                if str_vec.remove(0) != "->" {
                    return Commands::SyntaxError("Syntax: delete key -> 1 ".to_string());
                }
                let key_value = identify_value(str_vec.remove(0));
                return Commands::Delete(key_value);
            },
            "update" => {
                if str_vec.len() < 6 {
                    return Commands::SyntaxError("Syntax: update key -> 1 row -> blabalabala ".to_string());
                }
                let key_token = str_vec.remove(0);
                if key_token != "key" {
                    return Commands::SyntaxError("Syntax: update key -> 1 row -> blabalabala ".to_string());
                }
                if str_vec.remove(0) != "->" {
                    return  Commands::SyntaxError("Syntax: insert key -> 1 row -> blabalabala ".to_string());
                }
                let key_value = identify_value(str_vec.remove(0));
                if str_vec.remove(0) != "row" {
                    return Commands::SyntaxError("Syntax: update key -> 1 row -> blabalabala ".to_string());
                }
                if str_vec.remove(0) != "->" {
                    return Commands::SyntaxError("Syntax: update key -> 1 row -> blabalabala ".to_string());
                }
                let mut row = Vec::new();
                for types in column_types.unwrap() {
                    match types {
                        2 => {
                            let lexeme = str_vec.remove(0);
                            let number = lexeme.parse::<u32>().unwrap();
                            row.push(Value::Number(number));
                        }

                        7 => {
                            let lexeme = str_vec.remove(0);
                            row.push(Value::string(lexeme.to_string()))
                        }
                        _ => panic!()
                    }
                }

                return  Commands::Update(KeyRow { key: key_value, row: row }); 
            },
            "read" => {
                if str_vec.len() < 3 {
                    return Commands::SyntaxError("Syntax: read key -> 1 ".to_string());
                }
                let key_token = str_vec.remove(0);
                if key_token != "key" {
                    return Commands::SyntaxError("Syntax: read key -> 1 ".to_string());
                }
                if str_vec.remove(0) != "->" {
                    return Commands::SyntaxError("Syntax: read key -> 1 ".to_string());
                }
                let key_value = identify_value(str_vec.remove(0));
                return Commands::Read(key_value);
            },
            _ => return Commands::SyntaxError("Unknown Command token".to_string()),
         }
    };
    return Commands::SyntaxError("Empty input string".to_string());
}
pub fn transactions(file: &str, str: String,  trans: &mut bool) -> Response { 
    let tmp_path = "PlanckDB/tmp.db";

    let table = Table::read_table("tmp");
    
    let mut btree = BPlusTree::new(table);

    let command = parse(str, Some(&btree.buffer_pool.file.column_types));

    match command {
        Commands::CloseTransaction() => {
            fs::copy(tmp_path, format!("{}{}{}", "PlanckDB/", file, ".db")).unwrap();
            *trans = false;
            return Response::Query("Successfully saved the file and ended the transaction".to_string());
        }
        Commands::SyntaxError(x) => {
            return Response::Error(x);
        }
        Commands::Delete(x) => {
            delete(x, &mut btree);
            return Response::Query("Tried to delete a row".to_string());
        }
        Commands::Insert(x) => {
            insert(x, &mut btree);
            return Response::Query("Tried to insert a row".to_string());
        }
        Commands::Update(x) => {
            update(x, &mut btree);
            return Response::Query("Tried to insert a row".to_string());
        }
        Commands::Read(x) => {
            let result = read(x, &mut btree);
            match result {
                Err(_) => {
                    return Response::Error("Couldn't successfully find and read a row".to_string())
                }
                Ok(x) => {
                    return Response::Return(Packet{table: btree.buffer_pool.file.clone(), keyrows: vec![x]});
                }
            }
        }
        _ => {
            return Response::Error("You cannot start a transaction or create while another is open".to_string())
        }
    }
}
pub fn handle_client(mut stream: TcpStream) -> Result<(), Error>{
    let mut transaction = false;
    let mut file: String = "tmp".to_string();
    let mut buffer: [u8;4 ]= [0;4];
    loop {
        let mut payload: Vec<u8> = Vec::new();

        stream.read_exact(&mut buffer)?;
        let size = u32::from_be_bytes(buffer);
        payload.resize(size as usize, 0);
        stream.read_exact(&mut payload)?;

        let response = deserialize(&payload);

        match response {
            Response::Query(string) => {
                if transaction {
                    let response = transactions(&file, string,  &mut transaction);
                    
                    let bytes = serialize(response);

                    stream.write_all(&bytes)?;
                    
                }
                else {
                    let commands = parse(string, None);
                    match commands {
                        Commands::CloseTransaction() => break,
                        Commands::StartTransaction(x) => {
                            file = x;
                            transaction = true;
                            let tmp_path = "PlanckDB/tmp.db";
                            fs::copy(format!("{}{}{}", "PlanckDB/", file, ".db"), tmp_path).unwrap();
                            let bytes = serialize(Response::Query("Successfully opened the table".to_string()));
                            stream.write_all(&bytes).unwrap();
                        }, 
                        Commands::Create(file, pk, names, types) => {
                            match create(&file, pk, names, types) {
                                Ok(_) => {
                                    let bytes = serialize(Response::Query("Successfully created the table".to_string()));
                                    stream.write_all(&bytes).unwrap();
                                }
                                _ => {
                                    let bytes = serialize(Response::Error("While creating the file an error occurred, table might exists".to_string()));
                                    stream.write_all(&bytes).unwrap();
                                }
                            }
                        }
                        Commands::SyntaxError(x) => {
                            let bytes = serialize(Response::Error(x));
                            stream.write_all(&bytes)?;
                        }
                        _ => {
                            let bytes = serialize(Response::Error("You need to open a transaction to edit DB".to_string()));
                            stream.write_all(&bytes)?;
                        }
                    }
                }
            },
            _ => todo!()
        }
    }
    return Ok(())
}

pub fn server() {
    match fs::create_dir("PlanckDB") {
        Ok(_) => {
           
        }
        _ => {

        }
    }
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Listening at 127.0.0.1:7878");
    
    for stream in listener.incoming() {
        let safety = handle_client(stream.unwrap());
        match safety {
            Ok(_) => {

            }
            _ => eprintln!("Some kind of an error happened during data stream ? ")
        }
    }

}