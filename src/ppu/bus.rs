use mapper::Mapper;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct Bus {
    mapper: Weak<RefCell<Box<Mapper>>>,
}

impl Bus {
    pub fn new(mapper: &Rc<RefCell<Box<Mapper>>>) -> Self {
        Bus {
            mapper: Rc::downgrade(mapper),
        }
    }

    pub fn mapper(&self) -> Rc<RefCell<Box<Mapper>>> {
        self.mapper.upgrade().unwrap()
    }
}
