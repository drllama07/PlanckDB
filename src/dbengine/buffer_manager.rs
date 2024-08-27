use crate::dbengine::pages::*;
use crate::dbengine::engine::*;
use std::collections::HashMap;


const POOL_SIZE: usize = 6;

#[derive(Debug, Clone)]
pub struct Frame {
    page: Page,
    page_id: u32,
    pin: bool,
    dirty: bool,
}
#[derive(Debug, Clone)]
pub struct BufferPool {
    pub file: Table,
    pub pool: Vec<Frame>,
    pub table: HashMap<u32,usize>,
    clock: usize,

}

impl BufferPool {
    pub fn new(file: Table) -> Self {
        BufferPool {file: file,pool: Vec::new(), table: HashMap::new(), clock: 0}
    }

    fn get_index(&mut self, page_id: u32) -> Option<usize>{
        let mut index = 0;
        if self.table.contains_key(&page_id) {
            return Some(*self.table.get(&page_id).unwrap());
        }
        self.victim();
        self.load_from_disk(page_id);
        return Some(*self.table.get(&page_id).unwrap());
    }

    pub fn get_mut(&mut self, page_id: u32) -> &mut Page{
        let location = self.get_index(page_id).unwrap();
        let frame = &mut self.pool[location];
        frame.pin = true;
        frame.dirty = true;
        &mut frame.page
    }

    pub fn get(&mut self, page_id: u32) -> &Page{
        let location = self.get_index(page_id).unwrap();
        let frame = &mut self.pool[location];
        frame.pin = true;
        &frame.page
    }

    fn victim(&mut self) {
        if self.pool.len() == POOL_SIZE {    
            loop {
                // Get the frame at the current clock position
                let frame_index = self.clock % POOL_SIZE;
                let frame = &mut self.pool[frame_index];

                // Check if the frame is pinned
                if frame.pin {
                    frame.pin = false;
                    self.clock += 1;
                } else {
                    // If not pinned, we can evict this frame
                    // Remove the frame from the table
                    self.table.remove(&frame.page_id);
                    let frame = self.pool.remove(frame_index);
                    if frame.dirty {
                      self.flush_page(frame);  
                    }
                    // Update the clock position
                    self.clock = (self.clock + 1) % POOL_SIZE;
                    break;
                }


            }
        }
    }
    pub fn update_page(&mut self, page: Page,page_id: u32) {
        let file = &mut self.file;
        file.update_page(page.page_to_buff().unwrap(), page_id);
        if self.table.contains_key(&page_id) {
            self.pool[*self.table.get(&page_id).unwrap()].page = page;
        }
    }

    pub fn create_page(&mut self, page: Page) -> u32{
        let file = &mut self.file;
        let page_id = file.create_page(page.page_to_buff().unwrap());
        self.get_mut(page_id);
        return page_id;
    }

    pub fn remove_page(&mut self, page_id: u32) {
        if self.table.contains_key(&page_id) {
            let idx = self.table.remove(&page_id).unwrap();
            self.pool.remove(idx);
        }

        let mut index = 0;
        for frame in &self.pool {
            *self.table.get_mut(&frame.page_id).unwrap() = index;
            index += 1;
        }
        
        self.file.remove_page(page_id);
    }

    fn load_from_disk(&mut self , page_id: u32) {
       let buffer = self.file.read_page(page_id);
       let frame_new = Frame {page: Page::buff_to_page(self.file.column_types.clone(), buffer), page_id: page_id, pin: false, dirty: false};
       self.pool.push(frame_new);
       self.table.insert(page_id, 11);
       let mut index = 0;
       for frame in &self.pool {
           *self.table.get_mut(&frame.page_id).unwrap() = index;
           index += 1;
       }   
    }

    fn flush_page(&self, frame: Frame) {
       let table = &self.file;
       table.update_page(frame.page.page_to_buff().unwrap(), frame.page_id);
    }

    
    
    pub fn flush_all(&mut self) {
        let file = &self.file;
        for frame in &self.pool {
            if frame.dirty {
                file.update_page(frame.page.page_to_buff().unwrap(), frame.page_id);
            }
        }
    }
}


