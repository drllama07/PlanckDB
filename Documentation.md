<h1 align="center">
  <br>
  <a href="YOUR_REPO_LINK_HERE"><img src="https://github.com/drllama07/PlanckDB/blob/main/PLanck%20db.png" alt="YOUR_APP_NAME" width="200"></a>
  <br>
  Planck DB Documentation
  <br>
</h1>

<h4 align="center"></h4>
<p align="left" style="display: flex; flex-direction: column; align-items: left;">
  <br>
  Documentation:
  <br>
  <a href="#introduction">• Introduction</a> 
  <a href="#b-tree-implementation">• B+ Tree</a> 
  <a href="#buffer-manager">• Buffer Manager</a> 
  <a href="#disk-operations-and-file-format">• Disk Operations(I/O) and File Format</a> 
  <a href="#pages">• Pages</a> 
  <a href="#network-protocol">• Network Protocol</a>
  <a href="#demo-api">• Demo API Information</a>
</p>

# Introduction

The main goal of this documentation is to explain how Planck DB is implemented, making it easier for contributors to get involved or for those seeking inspiration to build something similar.

Each file has a dedicated section, which you can select from the list above.

If you are using this repository for learning, I highly suggest starting from [readme.md](README.md).

# B Tree Implementation
- Planck DB utilizes a B+ Tree variation for this project.
- Due to the complexity of the B+ Tree algorithm, I have created a separate repository dedicated to it.
- The B+ Tree algorithm in this repository includes some modifications to integrate it with the rest of the code. For a simpler version, check out my B+ Tree repository.
- I have discussed the conventions used in my B+ Tree implementation here -> [My B+ Tree Implementation](https://github.com/drllama07/B-Tree-in-rust?tab=readme-ov-file#implementation-details)

- ***RUST-NOTE:*** Given the nature of this project( main goal being getting better at Rust and DBs), `.clone()` is used frequently in the code. While this approach works, it may not be the most performance-efficient solution.

# Buffer Manager
- > ***The Main Structs***
  > ```rust
  > pub struct Frame {
  > page: Page,
  > page_id: u32,
  > pin: bool,
  > dirty: bool,
  > }
  > ```
  > Frame struct holds the pages and some additional information about the frame like pin state or page_id.
  > ```rust
  > pub struct BufferPool {
  >    pub file: Table,
  >    pub pool: Vec<Frame>,
  >    pub table: HashMap<u32,usize>,
  >    clock: usize,
  > }
  > ```
  > Buffer Pool struct holds the frames and the Table/File of the database.

- > ***The CLOCK Algorithm:***
  > Each time a page is requested and not found in our pool, we load it from disk. If the pool has reached its maximum size, we drop a page from the pool to make space.
  > `victim()` is the function that does it.
- > ***Helper Functions:***
  > These function are responsible for reading and writing pages to disk and setting pins and dirty flags fro the frames.

# Disk Operations and File Format
- > ***The Main Struct***
  > ```rust 
  > pub struct Table {
  > pub name: String,
  > pub pk_column: u8, // Not currently in use but I have some plans for it.
  > // Column number is actually the end of the byte sequence of column data. So sorry for the naming I was confussed too.
  > pub column_number: u8,
  > pub column_names: Vec<String>,
  > pub column_types: Vec<u8>,
  > pub page_id_count: u32,
  > pub root_node_offset: u32,
  > pub free_page_num: u32,
  > pub free_page_list: Vec<u32>
  > }
  > ``` 
  > Tables are stored in separate files, and each table has its own column types and names.
- > ***The File structure:***
  > Each file has a header containing information about that table like column types and free_page_list.
  > *Note:* header starts from 32th byte.
  > For more detailed look you can read `engine.rs`.
  > Page ids are the offset number at the same time. For example, page id 3 point to `3 * 4096` byte where the page starts.
- > ***Functions:***
  > This file handles the I/O operations like writing pages or reading tables.
  > There are function for reading, writing, and updating files.

# Pages
- As the pages physical structure, Planck DB adopts **slotted pages**.
- > ***Physical Structure of Pages:***
  > Each page consists of 4096 bytes.
  > The header of the page indicates whether the page is a leaf or an internal page(1 byte).
  > Next, we have the free space pointer (2 bytes).
  > Finally, the header includes the key type, which distinguishes between strings and u32 values (1 byte).
  > And the rest is slots and cells.
- > ***The Main Structs***
  > ```rust
  > pub enum Value {
  > Number(u32),
  > String(u8, String) // u8 is for the length of the string
  >}
  > ``` 
  > This is the core of Planck DB. This is the definition of a stored value.
  > ```rust 
  > pub struct Slot {
  > pub value: Value,
  > pub pointer: u16,
  >}
  >``` 
  > Slots are stored like this
  >```rust 
  > pub struct Rows {
  > pub size: u8,
  > pub values: Vec<Value>
  >}
  > ``` 
  > And this is the row struct.
  > ```rust
  > pub struct Page {
  > pub page_type: NodeType,
  > pub free_space_pointer: u16,
  > pub slots: Vec<Slot>,
  > pub cells: HashMap<u16,Rows>
  >}
  >``` 
  > And this is the in-memory structure of a page.
  > ```rust 
  > pub struct KeyRow {
  > pub key: Value,
  > pub row: Vec<Value>
  >}
  > ``` 
  > This is primarily used for transferring data throughout the program.
- > ***Functions:***
  > There are various functions for converting in-memory pages to byte arrays and for vacuuming pages to optimize space organization.

# Network Protocol
- > ***THE MAIN STRUCTS*** 
  > ```rust 
  > pub struct Packet {
  > pub table: Table,
  > pub keyrows: Vec<KeyRow>
  >}
  >``` 
  > This is the format that we transfer data between the server and the client.
  > ```rust 
  > pub enum Response {
  > Query(String),
  > Return(Packet),
  > Error(String),
  >}
  >```
  > These are the message types, each containing different data. This structure is particularly helpful for deconstructing messages.
- > Just like writing to disk we convert in-memory data structures in to byte arrays.
- > First 4 bytes contains the size of the message(array);
- > Next, the following byte indicates the message type (Response enum), and the remaining bytes involve converting between byte arrays and data structures.

# Demo API 

***WARNING:*** Due to corruption in my client folder, I was unable to upload it. However, if needed, I can either find a solution or rewrite the client code, as it is a small program.

### Features of this test app
1. Basic reading, writing, updating, and deleting rows.
2. Simple use interface with concrete syntax.
3. Commit or rollback control.
4. Simple error feedback.

### Executing Operations 
- `execute.rs` is the file that handles this job.
- This connects the storage engine to the use API.
- It is quite small program, understanding it shouldn't be a problem.

### Server
1. > ***Commands and Syntax***
   > ```
   > create FILE_NAME key -> KEY_TYPE(str or u32) columns | COLUMNS_1 |COLUMN_2 | ....
   > insert key -> THE_KEY row -> FIRST_COLUMN SECOND_COLUMN ... 
   > delete key -> THE_KEY
   > read key -> THE_KEY
   > update key -> THE_KEY row -> FIRST_COLUMN SECOND_COLUMN ... 
   > open FILE/TABLE_NAME
   > close 
   > exit
   > ```
   > Be careful while entering commands because each word is separated by empty space so don't do this `bla bal`, do this `bla_bal`.
   > `open` opens a transaction which allows for Planck DB to support rollback. For example, during the operation something happened and the operation was unsuccessful. When that happens the main file will be preserved and safe.
   > Only when you `close` the file it will be committed.
2. > ***Code Guide***
   > `transaction()` handles the executions and the temporary file operation for rollback.
   > `handle_client()` is the part where messages are interpreted and distributed to the right functions.
   > `server()` is wrapper function for all of this.


## In this documentation, I have highlighted only the most important aspects of the project. For more details, please refer to the code.
