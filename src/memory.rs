use crate::components::{Bus, Decoder8x256, Enableable, Register, Settable, Updatable};
use crate::gates::Wire;
use crate::gates::AND;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

pub const BUS_WIDTH: i32 = 16;

#[derive(Debug, Clone)]
pub struct Cell {
    pub value: Register,
    pub gates: [AND; 3],
}

impl Cell {
    fn new(input_bus: Rc<RefCell<Bus>>, output_bus: Rc<RefCell<Bus>>) -> Self {
        Self {
            value: Register::new("", input_bus, output_bus),
            gates: (0..3)
                .map(|_| AND::new())
                .collect::<Vec<AND>>()
                .try_into()
                .unwrap(),
        }
    }

    fn update(&mut self, set: bool, enable: bool) {
        self.gates[0].update(true, true);
        self.gates[1].update(self.gates[0].get(), set);
        self.gates[2].update(self.gates[0].get(), enable);

        match self.gates[1].get() {
            true => self.value.set(),
            false => self.value.unset(),
        }

        match self.gates[2].get() {
            true => self.value.enable(),
            false => self.value.disable(),
        }

        self.value.update()
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::new();
        for b in self.value.word.borrow().bits.iter() {
            match b.get() {
                true => res.push('1'),
                false => res.push('0'),
            }
        }
        write!(f, "{}", res)
    }
}

pub struct Memory64K {
    pub address_register: Rc<RefCell<Register>>,
    row_decoder: Decoder8x256,
    col_decoder: Decoder8x256,
    pub data: Vec<Vec<Cell>>,
    set: Wire,
    enable: Wire,
    pub bus: Rc<RefCell<Bus>>,
}

impl Memory64K {
    pub fn new(bus: Rc<RefCell<Bus>>) -> Self {
        Self {
            address_register: Rc::new(RefCell::new(Register::new("MAR", bus.clone(), bus.clone()))),
            row_decoder: Decoder8x256::new(),
            col_decoder: Decoder8x256::new(),
            data: (0..256)
                .map(|_| {
                    (0..256)
                        .map(|_| Cell::new(bus.clone(), bus.clone()))
                        .collect::<Vec<Cell>>()
                })
                .collect::<Vec<Vec<Cell>>>(),
            set: Wire::new("S".to_string(), false),
            enable: Wire::new("E".to_string(), false),
            bus,
        }
    }
}

impl Updatable for Memory64K {
    fn update(&mut self) {
        self.address_register.borrow_mut().update();
        self.row_decoder.update(
            self.address_register.borrow().bit(0),
            self.address_register.borrow().bit(1),
            self.address_register.borrow().bit(2),
            self.address_register.borrow().bit(3),
            self.address_register.borrow().bit(4),
            self.address_register.borrow().bit(5),
            self.address_register.borrow().bit(6),
            self.address_register.borrow().bit(7),
        );
        self.col_decoder.update(
            self.address_register.borrow().bit(8),
            self.address_register.borrow().bit(9),
            self.address_register.borrow().bit(10),
            self.address_register.borrow().bit(11),
            self.address_register.borrow().bit(12),
            self.address_register.borrow().bit(13),
            self.address_register.borrow().bit(14),
            self.address_register.borrow().bit(15),
        );

        self.data[self.row_decoder.index() as usize][self.col_decoder.index() as usize]
            .update(self.set.get(), self.enable.get())
    }
}

impl Enableable for Memory64K {
    fn enable(&mut self) {
        self.enable.update(true)
    }

    fn disable(&mut self) {
        self.enable.update(false)
    }
}

impl Settable for Memory64K {
    fn set(&mut self) {
        self.set.update(true)
    }

    fn unset(&mut self) {
        self.set.update(false)
    }
}

impl Display for Memory64K {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::from("Memory\n--------------------------------------\n");
        str.insert_str(
            str.len(),
            format!(
                "RD: {}\tCD: {}\tS: {}\tE: {}\t",
                self.row_decoder.index(),
                self.col_decoder.index(),
                self.set.get(),
                self.enable.get()
            )
            .as_str(),
        );

        for i in 0..256 {
            for j in 0..256 {
                let val = self.data[i][j].value.value();
                str.insert_str(str.len(), format!("0x{:04X}\t", val).as_str());
            }
        }
        str.insert_str(str.len(), "\n");

        write!(f, "{}", str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell() {
        let bus = Rc::new(RefCell::new(Bus::new(BUS_WIDTH)));
        let mut cell = Cell::new(bus.clone(), bus.clone());

        bus.borrow_mut().set_value(0xFFFF);
        cell.update(true, true);
        println!("{:?}", cell.value.value())
    }

    #[test]
    fn test_memory_64k_write() {
        let bus = Rc::new(RefCell::new(Bus::new(BUS_WIDTH)));
        let mut mem = Memory64K::new(bus.clone());

        let mut q: u16 = 0xFFFF;
        for i in 0x0000..0xFFFF {
            mem.address_register.borrow_mut().set();
            bus.borrow_mut().set_value(i);
            mem.update();

            mem.address_register.borrow_mut().unset();
            mem.update();

            bus.borrow_mut().set_value(q);

            mem.unset();
            mem.update();
            q -= 1;
        }

        let expected: u16 = 0xFFFF;
        for i in 0x0000..0xFFFF {
            mem.address_register.borrow_mut().set();
            bus.borrow_mut().set_value(i);
            mem.update();

            mem.address_register.borrow_mut().unset();
            mem.update();

            mem.enable();
            mem.update();

            mem.disable();
            mem.update();

            assert_eq!(bus.borrow().get_value(), expected);
        }

        println!("{}", mem);
    }
}
