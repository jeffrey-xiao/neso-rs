use mapper::Mapper;
use ppu::Ppu;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};

pub struct Bus {
    pub ppu: Weak<RefCell<Ppu>>,
    pub mapper: Weak<RefCell<Box<Mapper>>>,
}

impl Bus {
    pub fn new(ppu: &Rc<RefCell<Ppu>>, mapper: &Rc<RefCell<Box<Mapper>>>) -> Self {
        Bus {
            ppu: Rc::downgrade(ppu),
            mapper: Rc::downgrade(mapper),
        }
    }

    pub fn ppu(&self) -> Rc<RefCell<Ppu>> {
        self.ppu.upgrade().unwrap()
    }

    pub fn mapper(&self) -> Rc<RefCell<Box<Mapper>>> {
        self.mapper.upgrade().unwrap()
    }
}
