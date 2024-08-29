use std::{collections::HashMap, u32};
use std::mem;


use crate::dbengine::buffer_manager::*;
use crate::dbengine::pages::*;
use crate::dbengine::engine::Table;
#[derive(Clone, Debug)]
pub enum  NodeType {
    Internal(Vec<u32>),
    Leaf(Vec<u32>)
}
const BTREE_MAX: usize = 4;
const MIN_KEY: usize = BTREE_MAX/2;
const MAX_KEY: usize = BTREE_MAX;
const MIN_CHILD: usize = (BTREE_MAX+1)/2;
const MAX_CHILD: usize =BTREE_MAX+1;
const MAX_VALUE: u32 = u32::MAX;

#[derive(Clone, Debug)]
pub struct Siblings {
    page_id: u32 ,
    page_index: usize
}


#[derive(Clone, Debug)]
pub struct BPlusTree {
    pub buffer_pool: BufferPool,
}

impl BPlusTree {
    pub fn new(file: Table) -> Self {
        BPlusTree{buffer_pool: BufferPool::new(file)}
    }
    fn print_tree(&mut self, node_key: u32, level: usize) {
        if self.is_leaf_root() {
            println!("-> {:?}", self.buffer_pool.get(0))
        }
        else {
            if let page = self.buffer_pool.get(node_key) {
                // Print the current node with indentation
                let indent = "    ".repeat(level);
                match &page.page_type {
                    NodeType::Internal(kvs) => {
                        println!("{}Internal Node (Page_id: {}):", indent, node_key);
                        let slots = page.slots.clone();
                        let cells = page.cells.clone();
                        for slot in slots {
                            println!("{}  - Primary_index: {:?}, Points to Node: {:?}", indent, slot.value, cells.get(&slot.pointer).unwrap().values[0].extract_pointer());
                            // Recursively print the child nodes
                            self.print_tree(cells.get(&slot.pointer).unwrap().values[0].extract_pointer(), level + 1);
                        }
                    }
                    NodeType::Leaf(kvs) => {
                        println!("{}Leaf Node (Page_id: {}):", indent, node_key);
                        let slots = page.slots.clone();
                        let cells = page.cells.clone();
                        for slot in slots {
                            println!("{}  - Primary_index: {:?}, Rows: {:?}", indent, slot.value, cells.get(&slot.pointer).unwrap().values); 
                        }
                    }
                }
            }
         }
    }
    
    
    pub fn search(&mut self, k: &Value) -> (u32, u32){
        let root = self.buffer_pool.get(0).clone();
        self.search_tree(root, k, Some(0), Some(0))
    }
    fn search_tree(&mut self, node:Page, key: &Value, leaf_id: Option<u32>,parent_id: Option<u32>) -> (u32, u32){
       match node.page_type {
          NodeType::Leaf(keyvalue) => {
               return (leaf_id.unwrap(), parent_id.unwrap())
          },
          NodeType::Internal(keys) => {
                  let slots = &node.slots;
                  let mut internal_pointer: u16 = slots[node.slots.len() -1].pointer;
                  for i in 0..slots.len(){
                      if key < &slots[i].value {
                        internal_pointer = slots[i].pointer;
                        break;
                      }
                  };
                  let mut pointer = 0;
                  match node.cells.get(&internal_pointer).unwrap().values[0] {
                      Value::Number(x) => pointer = x,
                      _ => panic!()

                  }
                  
                  let child = self.buffer_pool.get(pointer).clone();
                  let mut parent:u32;
                  match &child.page_type {
                      NodeType::Internal(_) => parent = pointer,
                      NodeType::Leaf(_) => {
                        if parent_id != None {
                            parent = parent_id.unwrap();
                        } else {
                            parent = 0;
                        }
                      }
                  }
                  self.search_tree(child, key, Some(pointer), Some(parent))

          },
          _ => panic!("Undefined structure")
       }
    }

    fn is_underflow(&self, node:&Page) -> bool{
        match &node.page_type {
            NodeType::Internal(keys) => {
                if node.slots.len() < MIN_CHILD + 1 {
                    
                    return true
                } else {
                    return false 
                }
            },
            NodeType::Leaf(KeyValue) => {
                if node.slots.len() < MIN_KEY {
                    return true
                } else {
                    return false
                }
            }
        }
    }

    fn is_overflow(&self, node:&Page) -> bool{
        match &node.page_type {
            NodeType::Internal(keys) => {
                if node.slots.len() > MAX_CHILD {
                    return true
                } else {
                    return false 
                }
            },
            NodeType::Leaf(KeyValue) => {
                if node.slots.len() > MAX_KEY {
                    return true
                } else {
                    return false
                }
            }
        }
    }
    
    fn is_leaf_root(&mut self) -> bool {
        let root = self.buffer_pool.get(0);
        match &root.page_type {
            NodeType::Leaf(kvs) => {

                root.slots.len() < MAX_KEY + 1
            },
            _ => false,
        }
    }

    pub fn update(&mut self, new_kr: KeyRow) {
        let (node_id, _) = self.search(&new_kr.key);
        let node = self.buffer_pool.get_mut(node_id);
        node.delete(new_kr.key.clone());
        node.insert(new_kr);
    }

    fn insert_leaf_tree(&mut self, new_kr: KeyRow) {
        let root = self.buffer_pool.get_mut(0);
        root.insert(new_kr);
        let copy_cells = root.cells.clone();
        let copy_slots =  root.slots.clone();
        if !self.is_leaf_root() {
            
            let mut new_root = Page::new_internal();
            let mut new_page = Page::new_leaf();
            let str_num_spr = copy_slots[0].clone();
            new_page.slots = copy_slots;
            new_page.cells = copy_cells;
            let new_id = self.buffer_pool.create_page(new_page);
            match str_num_spr.value {
                Value::Number(_) =>  new_root.insert(KeyRow { key: Value::Number(MAX_VALUE), row: vec![Value::Number(new_id)] }),
                Value::String(_, _) => new_root.insert(KeyRow { key: Value::string("zzzzzzzz".to_string()), row: vec![Value::Number(new_id)] }),
            }
            self.buffer_pool.update_page(new_root, 0);
            self.split(new_id, 0);
        }
    }
    
    pub fn insert(&mut self, new_kr: KeyRow) -> bool {
        if self.is_leaf_root() {
            self.insert_leaf_tree(new_kr);
            return true;
        }
        let root = &self.buffer_pool.get(0).clone();
        let mut parents = vec![0];
        let mut next_node_pointer;
        match &root.page_type {
            NodeType::Internal(pkvs) => {
              let slots = &root.slots;
              next_node_pointer = slots[slots.len() -1].pointer;
              for i in 0..slots.len() - 1 {
                  if  new_kr.key < slots[i].value {
                      next_node_pointer = slots[i].pointer;
                      break;
                  }
              }
            },
            _ => panic!("__")
        }
        let next_node_id = &root.cells.get(&next_node_pointer).unwrap().values[0];
        self.insert_recursive(new_kr, next_node_id.extract_pointer(), &mut parents);
        while parents.len() > 1 {
            let node_id = parents.pop().unwrap();
            if self.is_overflow(self.buffer_pool.clone().get(node_id)) {
                self.split(node_id, parents[parents.len() -1]);
            } 
        }
        let root_id = parents.pop().unwrap();
        if self.is_overflow(self.buffer_pool.clone().get(root_id)) {
            self.split_root(root_id);
        }
        
        return true
        
    }

    fn insert_recursive(&mut self,new_kr: KeyRow,current: u32, parents: &mut Vec<u32>){
       let mut node = self.buffer_pool.get_mut(current);
       match &mut node.page_type {
        NodeType::Leaf(kvs) => {
            node.insert(new_kr);
            parents.push(current);
            
        },
        NodeType::Internal(kvs) => {
            let slots = &mut node.slots;
            parents.push(current);
            let mut next_node_id;
            next_node_id = slots[slots.len() -1].pointer;
            for i in 0..slots.len() - 1 {
                if  new_kr.key < slots[i].value {
                    next_node_id = slots[i].pointer;
                    break;
                }
            }
            let child = node.cells.get(&next_node_id).unwrap().values[0].clone();
            self.insert_recursive(new_kr, child.extract_pointer(), parents)
        }
       }

    }

    fn split_root(&mut self, root_id: u32) {
        let new_node_id = self.clone().buffer_pool.file.page_id_count;
        let root = self.buffer_pool.get_mut(root_id);
        let mut slots = &mut root.slots;
        let str_num_divider = slots[0].clone();
        let mut cells = &mut root.cells;
        let mut new_page = Page::new_internal();
        new_page.slots = mem::replace(&mut slots, Vec::new());
        new_page.cells = mem::replace(&mut cells, HashMap::new());
        
        let key = match str_num_divider.value {
            Value::Number(_) => Value::Number(MAX_VALUE),
            Value::String(_, _) => Value::string("zzzzzzzz".to_string()),
        };
        root.insert(KeyRow { key: key, row: vec![Value::Number(new_node_id)] });
        self.buffer_pool.create_page(new_page);
        self.split(new_node_id, root_id);
    }
    
    fn split(&mut self, current: u32, parent: u32) -> bool {
        let node = self.buffer_pool.get(current);
        let mut new_kr: Vec<KeyRow> = Vec::new();
        let mut is_internal = false;
        match &node.page_type {
          NodeType::Internal(kvs) => {
            let slots = &node.slots;
            for slot in slots {
                new_kr.push(KeyRow { key: slot.value.clone(), row: node.cells.get(&slot.pointer).unwrap().values.clone() })
            }
            is_internal = true;
          },
          NodeType::Leaf(kvs) => {
            let slots = &node.slots;
            for slot in slots {
                new_kr.push(KeyRow { key: slot.value.clone(), row: node.cells.get(&slot.pointer).unwrap().values.clone() })
            }
          }
        };
        let middle_index = new_kr.len() / 2;

        let mut new_node_vec: Vec<KeyRow> = Vec::new();
        for i in 0..middle_index {
            new_node_vec.push(new_kr[i].clone())
        }
        let mut divider = new_kr[middle_index].key.clone();
        if is_internal {
            divider = new_node_vec[middle_index-1].key.clone()
        }
        if is_internal{
            new_node_vec[middle_index-1].key = match new_node_vec[middle_index - 1].key {
                Value::Number(_) => Value::Number(MAX_VALUE),
                Value::String(_, _) => Value::string("zzzzzzzz".to_string()),
            };
        }
        let mut new_node_id = 0;
        let node = self.buffer_pool.get_mut(current);
        match &mut node.page_type {
            NodeType::Internal(kvs) => {
                for i in 0..middle_index {
                    let slot = node.slots.remove(0);
                    node.cells.remove(&slot.pointer);
                }
                let mut new_page = Page::new_internal();
                for i in 0..middle_index {
                    new_page.insert(new_node_vec.remove(0));
                }
                new_node_id = self.buffer_pool.create_page(new_page);
            },
            NodeType::Leaf(kvs) => {
                for i in 0..middle_index {
                    let slot = node.slots.remove(0);
                    node.cells.remove(&slot.pointer);
                }
                let mut new_page = Page::new_leaf();
                for i in 0..middle_index {
                    new_page.insert(new_node_vec.remove(0));
                }
                new_node_id = self.buffer_pool.create_page(new_page);
            }
        };

        
        let mut parent_node = self.buffer_pool.get_mut(parent);
        parent_node.insert(KeyRow { key: divider.clone(), row: vec![Value::Number(new_node_id)] });
        
       


       return  true
    }

    // Deletion Part

    pub fn delete(&mut self, key: Value) -> bool {
        let root = self.buffer_pool.get(0).clone();
        let mut parents = vec![0];
        let mut next_node_id;
        match root.page_type{
            NodeType::Internal(pkvs) => {
              let slots = root.slots;
              next_node_id = slots[slots.len() -1].pointer;
              for i in 0..slots.len() - 1 {
                  if  key < slots[i].value {
                      next_node_id = slots[i].pointer;
                      break;
                  }
              }
            },
            _ => panic!("__")
        }
        let pointer = root.cells.get(&next_node_id).unwrap().values[0].extract_pointer();
        let exists = self.delete_recursive(key, pointer, &mut parents);

        if exists {
            while parents.len() > 1 {
                let node_id = parents.pop().unwrap();
                if self.is_underflow(self.buffer_pool.clone().get(node_id)) {
                    self.distribute_mini(node_id, parents[parents.len() -1]);
                } 
            }
            self.merge_root(parents.pop().unwrap());
        }
        
        return true
        
    }

    
    fn get_sibling(&mut self , current: u32, parent: u32) -> Vec<Siblings>{
       let parent = self.buffer_pool.get(parent);
       match &parent.page_type {
        NodeType::Internal(pkvs) => {
            let slots = &parent.slots;
            let mut return_vec = Vec::new();
            let mut index_current = 0;
            for i in 0..slots.len() {
                let pointer = parent.cells.get(&slots[i].pointer).unwrap();
                if current ==  pointer.values[0].extract_pointer(){
                   index_current = i;
                   break;
                }
            }
            return_vec.push(Siblings { page_id: current, page_index: index_current});
            if index_current == 0 {
                return_vec.push(Siblings { page_id: parent.cells.get(&slots[1].pointer).unwrap().values[0].extract_pointer(), page_index: 1 });
            } else {
                return_vec.push(Siblings { page_id: parent.cells.get(&slots[index_current -1 ].pointer).unwrap().values[0].extract_pointer(), page_index: index_current -1  });
            }
            return return_vec;
        },
        _ => panic!("_+_")
       } 
    }

    fn delete_recursive(&mut self, key: Value, current: u32, parents: &mut Vec<u32>) -> bool {
        let mut node = self.buffer_pool.get_mut(current);
        match &mut node.page_type {
            NodeType::Leaf(kvs) => {
                let mut exists = false;
                let mut slots = &mut node.slots;
                for i in 0..slots.len() {
                    if key == slots[i].value {
                       let slot = slots.remove(i);
                       node.cells.remove(&slot.pointer);
                       node.vacuum();
                       parents.push(current); 
                       exists= true;
                       break;
                    }
                }
                if !exists {
                    println!("No key found as {:?}", key);
                }

                return exists;
            
            },
            NodeType::Internal(kvs) => {
                  parents.push(current);
                  let mut next_node_id;
                  let slots = &node.slots;
                  next_node_id = slots[slots.len() -1].pointer;
                  for i in 0..slots.len() - 1 {
                     if  key < slots[i].value {
                         next_node_id = slots[i].pointer;
                         break;
                     }
                  }
                  let pointer = node.cells.get(&next_node_id).unwrap().values[0].extract_pointer();
                  self.delete_recursive(key, pointer, parents)
            }
       }
    }


    // Merging Algorithm 
    fn distribute_mini(&mut self, current: u32, parent: u32) {
       let siblings = self.get_sibling(current, parent);
       let get_out = |vec: &Vec<Slot>, hash: &HashMap<u16, Rows>| -> Vec<KeyRow> {
             vec.iter().map(|item| KeyRow{key: item.value.clone(), row: hash.get(&item.pointer).unwrap().values.clone()}).collect()
        };
        
       let mut total_cells: Vec<KeyRow>= Vec::new();
       let mut reverse = false;
       let mut internal_divider: Value  = Value::Number(0);
       if siblings[0].page_index < siblings[1].page_index {
          let node1 = self.buffer_pool.get(siblings[0].page_id).clone();
          let node2 = self.buffer_pool.get(siblings[1].page_id).clone();
          match &node1.page_type {
            NodeType::Internal(kvs) => {
                
                let slots = &node1.slots;
                let cells = &node1.cells;
                total_cells.extend(get_out(slots, cells));

                let parent = self.buffer_pool.get(parent);
                match &parent.page_type {
                    NodeType::Internal(pkvs) => {
                        internal_divider = parent.slots[siblings[0].page_index].value.clone();
                    },
                    _ => panic!("-")
                }
            },
            NodeType::Leaf(kvs) => {
                let slots = &node1.slots;
                let cells = &node1.cells;
                total_cells.extend(get_out(slots, cells));
            },  
          };
          match &node2.page_type {
            NodeType::Internal(kvs) => {
                let slots = &node2.slots;
                let cells = &node2.cells;
                total_cells.extend(get_out(slots, cells));
            },
            NodeType::Leaf(kvs) => {
                let slots = &node2.slots;
                let cells = &node2.cells;
                total_cells.extend(get_out(slots, cells));
            },
        }
       } else {
            reverse = true;
            
            let node1 = self.buffer_pool.get(siblings[0].page_id).clone();
            let node2 = self.buffer_pool.get(siblings[1].page_id).clone();
            match &node2.page_type {
                NodeType::Internal(kvs) => {
                    let slots = &node2.slots;
                    let cells = &node2.cells;
                    total_cells.extend(get_out(slots, cells));


                    let parent = self.buffer_pool.get(parent);
                    match &parent.page_type {
                        NodeType::Internal(pkvs) => {
                            internal_divider = parent.slots[siblings[1].page_index].value.clone();
                        },
                        _ => panic!("-")
                    }
                    
                },
                NodeType::Leaf(kvs) => {
                    let slots = &node2.slots;
                    let cells = &node2.cells;
                    total_cells.extend(get_out(slots, cells));
                },  
            };
            match &node1.page_type {
                NodeType::Internal(kvs) => {
                    let slots = &node1.slots;
                    let cells = &node1.cells;
                    total_cells.extend(get_out(slots, cells));
                },
                NodeType::Leaf(kvs) => {
                    let slots = &node1.slots;
                    let cells = &node1.cells;
                    total_cells.extend(get_out(slots, cells));
                },
            }
       }
       
       
       if reverse {
        let node2 = self.buffer_pool.get_mut(siblings[1].page_id);
        match &mut node2.page_type {
           NodeType::Leaf(kvs) => {
            
             if total_cells.len() <= MAX_KEY {
                let node1 = self.buffer_pool.get_mut(siblings[0].page_id);
                 match &mut node1.page_type {
                     NodeType::Leaf(kvs1) => {
                        node1.clean_page();
                        
                        for kr in total_cells {
                            node1.insert(kr);
                        }
                     },
                     _ => panic!("___")
                 }
                 self.buffer_pool.remove_page(siblings[1].page_id);
                 let parent_node = self.buffer_pool.get_mut(parent);
                 match &mut parent_node.page_type {
                     NodeType::Internal(pkvs) => {
                         let slot = parent_node.slots.remove(siblings[1].page_index);
                         parent_node.cells.remove(&slot.pointer);
                     },
                     _ => panic!("___")
                 }
             } else {
                 let mut moved_slot = node2.slots.pop().unwrap();
                 let mut moved_value = KeyRow {key: moved_slot.value, row: node2.cells.remove(&moved_slot.pointer).unwrap().values};
                 let last_idx = &node2.slots.len() -1;
                 let new_bound = moved_value.key.clone();
                 let node1 = self.buffer_pool.get_mut(siblings[0].page_id);
                 node1.insert(moved_value);
                 let parent_node = self.buffer_pool.get_mut(parent);
                 match &mut parent_node.page_type {
                     NodeType::Internal(pkvs) => {
                         parent_node.slots[siblings[1].page_index].value = new_bound;
                     },
                     _ => panic!("___")
                 }
             }
           },
           NodeType::Internal(kvs) => {
            
             if total_cells.len() <= MAX_CHILD {
                 for i in 0..total_cells.len() {
                     match  &total_cells[i].key{
                         Value::Number(x) => {
                            if x.clone() == MAX_VALUE {
                                total_cells[i].key = internal_divider;
                                break;
                            }
                         },
                         Value::String(_, x) => {
                            if x.clone() == "zzzzzzzz".to_string(){
                                total_cells[i].key = internal_divider;
                                break;
                            }
                         }
                     }
                 }
                 let node1 = self.buffer_pool.get_mut(siblings[0].page_id);
                 node1.clean_page();
                 for kr in total_cells {
                    node1.insert(kr)
                 }
                 self.buffer_pool.remove_page(siblings[1].page_id);
                 let parent_node = self.buffer_pool.get_mut(parent);
                 match &mut parent_node.page_type {
                     NodeType::Internal(pkvs) => {
                         
                        let slot = parent_node.slots.remove(siblings[1].page_index);
                        parent_node.cells.remove(&slot.pointer);
                     },
                     _ => panic!("___")
                 }
                 
             } else {
                 let slots = &mut node2.slots;
                 let mut moved_slot = slots.pop().unwrap();
                 let mut moved_value = KeyRow{key: moved_slot.value, row: node2.cells.remove(&moved_slot.pointer).unwrap().values};
                 let last_idx = &slots.len() -1;
                 let new_bound = slots[last_idx].value.clone();
                 match &mut slots[last_idx].value{
                     Value::Number(_) => slots[last_idx].value = Value::Number(MAX_VALUE),
                     Value::String(_, _) => slots[last_idx].value = Value::string("zzzzzzzz".to_string())
                 }
                 moved_value.key = internal_divider;
                 let node1 = self.buffer_pool.get_mut(siblings[0].page_id);
                 node1.insert(moved_value);
                 let parent_node = self.buffer_pool.get_mut(parent);
                 match &mut parent_node.page_type {
                     NodeType::Internal(pkvs) => {
                         parent_node.slots[siblings[1].page_index].value = new_bound;
                     },
                     _ => panic!("___")
                 }
             }
           }
        }
       } else {
           let node2 = self.buffer_pool.get_mut(siblings[1].page_id);
           match &mut node2.page_type {
              NodeType::Leaf(kvs) => {
                if total_cells.len() <= MAX_KEY {
                    node2.clean_page();
                    for kr in total_cells {
                        node2.insert(kr);
                    };

                    self.buffer_pool.remove_page(siblings[0].page_id);
                    let parent_node = self.buffer_pool.get_mut(parent);
                    match &mut parent_node.page_type {
                        NodeType::Internal(pkvs) => {
                            let slot = parent_node.slots.remove(siblings[0].page_index);
                            parent_node.cells.remove(&slot.pointer);
                        },
                        _ => panic!("___")
                    }
                } else {
                    let slots = &mut node2.slots;
                    let moved_slot = slots.remove(0);
                    let moved_value = KeyRow{key: moved_slot.value, row: node2.cells.remove(&moved_slot.pointer).unwrap().values};
                    let new_bound = slots[0].value.clone();
                    let node1 = self.buffer_pool.get_mut(siblings[0].page_id);
                    node1.insert(moved_value);
                    let parent_node = self.buffer_pool.get_mut(parent);
                    match &mut parent_node.page_type {
                        NodeType::Internal(pkvs) => {
                            parent_node.slots[siblings[0].page_index].value = new_bound;
                        },
                        _ => panic!("___")
                    }
                }
              },

              NodeType::Internal(kvs) => {
                if total_cells.len() <= MAX_CHILD {
                    for i in 0..total_cells.len() {
                        match  &total_cells[i].key {
                            Value::Number(x) => {
                               if x.clone() == MAX_VALUE as u32 {
                                   total_cells[i].key = internal_divider;
                                   break;
                               }
                            },
                            Value::String(_, x) => {
                               if x.clone() == "zzzzzzzz".to_string(){
                                   total_cells[i].key = internal_divider;
                                   break;
                               }
                            }
                        }
                    }
                    node2.clean_page();
                    for kr in total_cells{
                        node2.insert(kr)
                    }
                    self.buffer_pool.remove_page(siblings[0].page_id);
                    let parent_node = self.buffer_pool.get_mut(parent);
                    match &mut parent_node.page_type {
                        NodeType::Internal(pkvs) => {
                            let slot = parent_node.slots.remove(siblings[0].page_index);
                            parent_node.cells.remove(&slot.pointer);
                            
                        },
                        _ => panic!("___")
                    }
                    
                } else {
                    let slots = &mut node2.slots;
                    let mut moved_slot = slots.remove(0);
                    let mut moved_value = KeyRow{key: moved_slot.value, row: node2.cells.remove(&moved_slot.pointer).unwrap().values};
                    let new_bound = moved_value.key.clone();
                    match &moved_value.key {
                        Value::Number(_) => moved_value.key = Value::Number(MAX_VALUE),
                        Value::String(_, _) => moved_value.key = Value::string("zzzzzzzz".to_string()),
                    }
                    let node1 = self.buffer_pool.get_mut(siblings[0].page_id);
                    match &mut node1.page_type {
                        NodeType::Internal(kvs1) => {
                            let slots = &mut node1.slots;
                            let last_idx = &slots.len() -1;
                            slots[last_idx].value = internal_divider;
                            
                        },
                        _ => panic!("___")
                    }
                    node1.insert(moved_value);

                    let parent_node = self.buffer_pool.get_mut(parent);
                    match &mut parent_node.page_type {
                        NodeType::Internal(pkvs) => {
                            let slots = &mut parent_node.slots;
                            slots[siblings[0].page_index].value = new_bound;
                        },
                        _ => panic!("___")
                    }
                }
              }
           }
       }
       
       
     }
     
     fn merge_root(&mut self, root_id: u32) -> bool{
        let get_out = |vec: &Vec<Slot>, hash: &HashMap<u16, Rows>| -> Vec<KeyRow> {
            vec.iter().map(|item| KeyRow{key: item.value.clone(), row: hash.get(&item.pointer).unwrap().values.clone()}).collect()
        };
        let root = self.buffer_pool.get(root_id);
        let mut child_id: u32 = 0;
        let mut cells: Vec<KeyRow> = Vec::new();
        match &root.page_type {
         NodeType::Internal(pkvs) => {
            let slots = &root.slots;
            if slots.len() > 1 {
                return false
            }
             child_id = root.cells.get(&slots[0].pointer).unwrap().values[0].extract_pointer();
         },
         _ => panic!("__")
        };
        let child = self.buffer_pool.get(child_id);
        match &child.page_type {
          NodeType::Internal(kvs) => {
             let output = get_out(&child.slots, &child.cells);
             cells.extend(output.into_iter());
          },
          NodeType::Leaf(kvs) => {
             return false
          }
        };
 
        self.buffer_pool.remove_page(child_id);
 
        let root = self.buffer_pool.get_mut(root_id);
        root.clean_page();
        for kr in cells {
            root.insert(kr)
        }
        true
     }
    

}
