use super::{Component, Wire, NAND};

#[derive(Debug, Clone)]
struct Bit {
    gates: [NAND; 4],
    wire_o: Wire,
}

impl Bit {
    fn new() -> Self {
        Self {
            gates: (0..4)
                .map(|_| NAND::new())
                .collect::<Vec<NAND>>()
                .try_into()
                .unwrap(),
            wire_o: Wire::new("O".to_string(), false),
        }
    }

    fn get(&self) -> bool {
        self.wire_o.get()
    }

    fn update(&mut self, wire_i: bool, wire_s: bool) {
        (0..2).map(|_| {
            self.gates[0].update(wire_i, wire_s);
            self.gates[1].update(self.gates[0].get(), wire_s);
            self.gates[2].update(self.gates[0].get(), self.gates[3].get());
            self.gates[3].update(self.gates[2].get(), self.gates[1].get());
            self.wire_o.update(self.gates[2].get());
        });
    }
}

#[derive(Clone)]
pub struct Bit16 {
    inputs: [Wire; 16],
    bits: [Bit; 16],
    outputs: [Wire; 16],
    next: Option<Box<dyn Component>>,
}

impl Bit16 {
    pub fn new() -> Self {
        Self {
            inputs: (0..16)
                .map(|_| Wire::new("Z".to_string(), false))
                .collect::<Vec<Wire>>()
                .try_into()
                .unwrap(),
            bits: (0..16)
                .map(|_| Bit::new())
                .collect::<Vec<Bit>>()
                .try_into()
                .unwrap(),
            outputs: (0..16)
                .map(|_| Wire::new("Z".to_string(), false))
                .collect::<Vec<Wire>>()
                .try_into()
                .unwrap(),
            next: None,
        }
    }

    pub fn update(&mut self, set: bool) {
        for i in 0..self.inputs.len() {
            self.bits[i].update(self.inputs[i].get(), set);
            self.outputs[i].update(self.bits[i].get());
        }

        match self.next.as_mut() {
            Some(next) => {
                for i in 0..self.outputs.len() {
                    next.set_input_wire(i as i32, self.outputs[i].get());
                }
            }
            _ => {}
        };
    }
}

impl Component for Bit16 {
    fn set_input_wire(&mut self, i: i32, value: bool) {
        self.inputs[i as usize].update(value)
    }
    fn get_output_wire(&self, i: i32) -> bool {
        self.outputs[i as usize].get()
    }
}
