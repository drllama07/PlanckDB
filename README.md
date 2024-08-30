<h1 align="center">
  <br>
  <a href="YOUR_REPO_LINK_HERE"><img src="https://github.com/drllama07/PlanckDB/blob/main/PLanck%20db.png" alt="YOUR_APP_NAME" width="200"></a>
  <br>
  Planck DB 
  <br>
</h1>

<h4 align="center"></h4>
<p align="center">
  <a href="#what-is-mka-?">How</a> •
  <a href="#installation">Installation</a> •
  <a href="#starter-guide">Starter Guide</a> •
  <a href="#code-guide">Code Guide</a> •
  <a href="#learning-resources-&-credits">Learning Resources</a> •
  <a href="#license">License</a>
</p>

# Planck DB ? 
Planck DB is a disk-persistent key-value store and toy database, designed for learning purposes and small project implementations.

- Disk-persistent

- Supports unsigned 32-bit integers and variable-length strings

- Similar to relational databases, you can store multiple values in rows, and for keys, you can choose either a string or a u32

- Includes a minimal network protocol for flexibility and a small CLI text app for testing
- For fast indexing, Planck DB leverages a B+ tree implementation

- Implements the "CLOCK" memory management algorithm to minimize memory usage

- For more reliable operations, Planck DB includes a safety mechanism similar to rollback operations

> ## Warning 
> Even if it is designed for learning purposes, my code might not be optimal or sufficient, especially considering my level of experience. Take this code with a grain of salt and be aware of the lack of thorough testing and optimization. I appreciate any feedback for improvement.

## How could someone make use out of this code ?
> If you want to learn more about databases, I'll share the learning resources I've used and some of my experiences in the[Learning Resources](README.md/credits-and-learning-resources) section.

> If you design small project and need a simple persistent storage. You can leverage and if possible adjust Planck DB for your needs.

> Or just as a starting point for your own database implementation.

# Installation

1. Download the repository.

> **Note:** If you are not familiar with programming in Rust, feel free to reach out to me, and I will try my best to help you.


2. Build the executable using [Rust's Cargo](https://www.rust-lang.org/tools/install).

```bash
cargo build
```
> For more info on how to build and use executables you can search online

3. ```bash
   cd target\debug
   your_executable_name.exe
   ```
4. You must do this for both `/client` and the main code.


# For more information on how to use PLanck DB and for more implementation details:
->  [PlanckDB-Documentation](documentation.md)

# Projects ideas for testing Planck DB

> ***A simple password manager app:***

> Key => Website Address 

> Row => Column 1 -> Username  | Column 2 -> Password

> All of the values stored will be strings(7) for this case.

# *Code Guide*

*Side-Note:* Here, I will be merely explaining the code and CS concepts, not the implementation details that could be useful for using Planck DB. For that, refer to the [PlanckDB-Documentation](documentation.md)

## Storage Engine and Memory Manager (`/dbengine`):

**Disk Operations (I/O)**
> *engine.rs* and *pages.rs*
- These files are responsible for converting in-memory data structures to byte arrays and then writing them to disk.

### Writing to the disk (`engine.rs`)
- This file handles the I/O operations by leveraging [Rust's I/O capabilities](https://doc.rust-lang.org/std/fs/index.html)
- Tables are defined here, and each table is written to its own file
- This file is also responsible for writing and updating the pages (which I will describe later)
- In this file, we only convert `Table` data structure into bytes

### Dealing with the Pages (`pages.rs`)
- The main objective of this file is to do all kinds of operations to `Page` in-memory data structure. 
- Here is a short list of the operations: 
- 1. Contains the `Page` struct and several sub-structs used throughout the project.
  2. Converting `Page` structs into byte arrays.
  3. Implements insert, delete, and update like functions to pages.
  4. And more...

**In-Memory Operations**
> *btrees.rs* and *buffer_manager.rs*
- These files are responsible for doing the in-memory work.

### Managing the Precious Memory (`buffer_manager.rs`)
- For many databases, retrieving large chunks of bytes into memory is challenging due to the memory size limitations.
- Therefore, similar to larger databases, Planck DB uses a memory management algorithm called `CLOCK`.
- Here, we manage the retrieval of pages and determine when to write them back to disk.
- This allows us to reduce I/O and operational costs.

### How to find the indexes fast (`btrees.rs`)
- It is common for databases to use a B-Tree variation for indexing. An example would be SQLite.
- This file is all about my B+ Tree implementation for pages.

## User API (`/TCP_connections`)

This folder contains the code responsible for the TCP connections (client/server) and some related utilities.

### Making B+ Tree more accessible (`execute.rs`)
- This adds a new layer to the B+ Tree API, making it more useful for database queries.
- Here, we have functions for reading, writing, and performing other operations on a row.

### A Simple Network Protocol for Planck DB (`protocols.rs`)
- Just like writing to disk, we need to convert in-memory data structures into bytes.
- Thus, this file implements a simple network protocol.

### A Tiny Server (`server.rs`) & (`client.rs`)
- Using [Rust's networking libraries](https://doc.rust-lang.org/std/net/), we have created a small server and a client for testing Planck DB and its network protocols.


# Learning Resources & Credits

 **A Great Example Project:** 
   > This project is a Database implementation by [Tony Saro](https://youtu.be/5Pc18ge9ohI?si=rbhZjVrmVdmZteby). He has created a video series that provides an explanation of it.

 **Outstanding Lectures Offered for Free**
 > The CMU Database Group offers excellent [introduction to databases](https://www.youtube.com/watch?v=vdPALZ-GCfI&list=PLSE8ODhjZXjbj8BMuIrRcacnQh20hmY9g) lectures and [coursework](https://15445.courses.cs.cmu.edu/fall2021/assignments.html) that anyone can use.

 ### B+ Trees
 - Personally, implementing a B+ tree was the most time- and energy-consuming part of this project, so I have created a separate repository for [my B+ tree implementation](https://github.com/drllama07/B-Tree-in-rust) along with some other learning resources.
 - 1. https://www.db-book.com/slides-dir/PDF-dir/ch14.pdf
   2. https://sqlite.org/btreemodule.html
   3. https://15445.courses.cs.cmu.edu/fall2020/project2/
   4. https://github.com/nimrodshn/btree/blob/master/src/btree.rs
   
### Buffer Manager(Memory Managment)
- For a simple implementation this should be enough -> 
https://www.youtube.com/watch?v=Y9H2HaRKOIw&t=1523s

### Physical Storage Methods, Including Pages and More...
- https://www.db-book.com/slides-dir/PDF-dir/ch13.pdf
- https://www.db-book.com/slides-dir/PDF-dir/ch12.pdf
- https://youtu.be/DJ5u5HrbcMk?si=oazQRgMP4QEOw4ex 
- And other lectures on this topic.
### Networking and Protocols
- https://youtu.be/LPxALdezA8Y?si=z0uQkeJ02YhZmVY8
- https://youtu.be/f-KltQKLwd0?si=4eRHuq_nC0V2OGQ9

## For More Learning Resources like these

Check out my [X](https://x.com/lab_of_learning) account, where I post learning resources on various subjects, including computer science.

You may also like this -> [MY MKA Programming Language](https://github.com/drllama07/MKA_Programming_language)


# Personal Code Review
## Project Goal
I wanted to create a small database that could be used for projects I sometimes work on. While developing it, I explored lower-level programming concepts such as bit operations, hardware I/O, and complex data structures and algorithms like B+ Trees. The biggest challenge of this project was debugging byte-level operations, which took a lot of time as the project grew. I feel that my code needs to improve in terms of performance and maintainability. My next goal is to work on projects with better coding practices.
 
## Programming Review
- > **Got even better at Rust**
- > - I/O library 
  > - Network library
  > - Rust's memory management like borrow checker and life times
  > - Unfortunately a lot of `.clone()`
  > - Recursion

- > **Computer Science**
  > - Bytes and bits 
  > - Memory and disk
  > - B+ Tree
  > - Network Protocols
  > - Database Concepts and types



# License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
