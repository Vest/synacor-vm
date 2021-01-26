use crate::mem::{Memory, MAX_ADDRESS};
use std::cell::RefCell;
use std::rc::Rc;

const MAX_REGISTERS: usize = 8;

#[derive(Debug)]
pub enum CPUError {
    OverflowAddress(u16),
    OverflowRegister(u8),
    UnknownOpCode { opcode: u16, address: u16 },
}

enum ExecutionResult {
    Stop,
    Jump(u16),
    Next(u16),
}

pub struct CPU {
    memory: Rc<RefCell<Memory>>,
    registers: [u16; MAX_REGISTERS],
    stack: Vec<u16>,

    current_address: u16,
}

impl CPU {
    pub fn new(mem: Rc<RefCell<Memory>>) -> CPU {
        CPU {
            memory: mem,
            registers: [0; MAX_REGISTERS],
            stack: Vec::new(),

            current_address: 0,
        }
    }

    pub fn dump_cpu(&self) {
        println!("--- Registers ---");
        self.registers.iter()
            .enumerate()
            .for_each(|(idx, value)| {
                println!("reg {}: {:#06X}", idx, value);
            });
        println!();
        println!("--- Stack ---");
        println!("top ---");
        self.stack.iter()
            .rev()
            .for_each(|value| {
                println!("{:#06X}", value);
            });
        println!("--- bottom");
        /*
                println!("--- Memory ---");
                let mem = self.memory.borrow();
                (0..0x8000_u16).for_each(|address| {
                    if address % 16 == 0 { println!(); }
                    if address % 0x1000 == 0 { println!("--------------------------"); }
                    print!("{:#06X} ", mem.read_memory(address).unwrap());
                });

         */
    }

    pub fn get_value_from_address(&self, address: u16) -> Result<u16, CPUError> {
        match address {
            0..=0x7FFF => {
                self.memory.borrow()
                    .read_memory(address)
                    .ok_or(CPUError::OverflowAddress(address))
            }
            0x8000..=0x8007 => {
                let reg_num = get_registry_from_address(address)
                    .ok_or(CPUError::OverflowAddress(address))?;

                self.read_register(reg_num)
                    .ok_or(CPUError::OverflowRegister(reg_num))
            }
            _ => Err(CPUError::OverflowAddress(address)),
        }
    }

    pub fn set_value_in_address(&mut self, address: u16, value: u16) -> Result<u16, CPUError> {
        match address {
            0..=0x7FFF => {
                self.memory.borrow_mut()
                    .write_memory(address, value)
                    .or(Err(CPUError::OverflowAddress(address)))
            }
            0x8000..=0x8007 => {
                let reg_num = get_registry_from_address(address)
                    .ok_or(CPUError::OverflowAddress(address))?;
                self.write_register(reg_num, value)
            }
            _ => Err(CPUError::OverflowAddress(address)),
        }
    }

    pub fn read_register(&self, number: u8) -> Option<u16> {
        match number {
            0..=7 => Some(self.registers[number as usize]),
            _ => None,
        }
    }

    pub fn write_register(&mut self, number: u8, value: u16) -> Result<u16, CPUError> {
        match number {
            0..=7 => {
                let old_value = self.registers[number as usize];
                self.registers[number as usize] = value;

                Ok(old_value)
            }
            _ => Err(CPUError::OverflowRegister(number)),
        }
    }

    fn from_raw_to_u16(&self, raw: u16) -> Result<u16, CPUError> {
        match raw {
            0..=0x7FFF => Ok(raw),
            0x8000..=0x8007 => {
                let reg_num = get_registry_from_address(raw)
                    .ok_or(CPUError::OverflowAddress(raw))?;

                self.read_register(reg_num)
                    .ok_or(CPUError::OverflowRegister(reg_num))
            }
            _ => Err(CPUError::OverflowAddress(raw)),
        }
    }

    pub fn pop(&mut self) -> Option<u16> {
        self.stack.pop()
    }

    pub fn execute(&mut self) -> Result<(), CPUError> {
        loop {
            let op_code = self.get_value_from_address(self.current_address)?;
            let a = self.get_value_from_address(self.current_address + 1);
            let b = self.get_value_from_address(self.current_address + 2);
            let c = self.get_value_from_address(self.current_address + 3);

            let execution_result = match op_code {
                0 => self.halt(),
                1 => self.set(a?, b?),
                2 => self.push(a?),
                4 => self.eq(a?, b?, c?),
                6 => self.jmp(a?),
                7 => self.jt(a?, b?),
                8 => self.jf(a?, b?),
                9 => self.add(a?, b?, c?),
                19 => self.out(a?),
                21 => self.noop(),

                _ => Err(CPUError::UnknownOpCode {
                    opcode: op_code,
                    address: self.current_address,
                }),
            };

            match execution_result? {
                ExecutionResult::Stop => break,
                ExecutionResult::Jump(address) => self.current_address = address,
                ExecutionResult::Next(size) => self.current_address += size,
            }
        }

        Ok(())
    }

    // halt: 0 - stop execution and terminate the program
    fn halt(&self) -> Result<ExecutionResult, CPUError> {
        Ok(ExecutionResult::Stop)
    }

    // set: 1 a b - set register <a> to the value of <b>
    fn set(&mut self, raw_a: u16, b: u16) -> Result<ExecutionResult, CPUError> {
        let _ = self.set_value_in_address(raw_a, b)?;

        Ok(ExecutionResult::Next(3))
    }

    // push: 2 a - push <a> onto the stack
    fn push(&mut self, raw_a: u16) -> Result<ExecutionResult, CPUError> {
        let a = self.from_raw_to_u16(raw_a)?;
        self.stack.push(a);

        Ok(ExecutionResult::Next(2))
    }

    // eq: 4 a b c - set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
    fn eq(&mut self, raw_a: u16, raw_b: u16, raw_c: u16) -> Result<ExecutionResult, CPUError> {
        let b = self.from_raw_to_u16(raw_b)?;
        let c = self.from_raw_to_u16(raw_c)?;

        if b == c {
            self.set_value_in_address(raw_a, 1)?;
        } else {
            self.set_value_in_address(raw_a, 0)?;
        }
        Ok(ExecutionResult::Next(4))
    }


    // jmp: 6 a - jump to <a>
    fn jmp(&self, a: u16) -> Result<ExecutionResult, CPUError> {
        Ok(ExecutionResult::Jump(a))
    }

    // jt: 7 a b - if <a> is nonzero, jump to <b>
    fn jt(&self, raw_a: u16, b: u16) -> Result<ExecutionResult, CPUError> {
        let a = self.from_raw_to_u16(raw_a)?;

        Ok(if a != 0 {
            ExecutionResult::Jump(b)
        } else {
            ExecutionResult::Next(3)
        })
    }

    // jf: 8 a b - if <a> is zero, jump to <b>
    fn jf(&self, raw_a: u16, b: u16) -> Result<ExecutionResult, CPUError> {
        let a = self.from_raw_to_u16(raw_a)?;

        Ok(if a == 0 {
            ExecutionResult::Jump(b)
        } else {
            ExecutionResult::Next(3)
        })
    }

    // add: 9 a b c - assign into <a> the sum of <b> and <c> (modulo 32768)
    fn add(&mut self, raw_a: u16, b: u16, c: u16) -> Result<ExecutionResult, CPUError> {
        let sum = b.wrapping_add(c);

        self.set_value_in_address(raw_a, sum)?;
        Ok(ExecutionResult::Next(4))
    }

    // out: 19 a - write the character represented by ascii code <a> to the terminal
    fn out(&self, a: u16) -> Result<ExecutionResult, CPUError> {
        let a = (a as u8) as char;

        print!("{}", a);

        Ok(ExecutionResult::Next(2))
    }

    // noop: 21 - no operation
    fn noop(&self) -> Result<ExecutionResult, CPUError> {
        Ok(ExecutionResult::Next(1))
    }
}

fn get_registry_from_address(address: u16) -> Option<u8> {
    if let Some(reg_num) = address.checked_sub(MAX_ADDRESS as u16) {
        if (0..MAX_REGISTERS as u16).contains(&reg_num) {
            return Some(reg_num as u8);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_registry_from_address() {
        assert_eq!(get_registry_from_address(0), None);
        assert_eq!(get_registry_from_address(32768), Some(0));
        assert_eq!(get_registry_from_address(32769), Some(1));
        assert_eq!(get_registry_from_address(32774), Some(6));
        assert_eq!(get_registry_from_address(32775), Some(7));
        assert_eq!(get_registry_from_address(34776), None);
        assert_eq!(get_registry_from_address(u16::MAX), None);
    }

    #[test]
    fn test_set_value() {
        let mut mem = Memory::default();
        mem.load_data(&[3, 2, 1]).ok();

        let mut cpu = CPU::new(Rc::new(RefCell::new(mem)));
        let old_value = cpu.set_value_in_address(0, 0).unwrap_or(u16::MAX);

        assert_eq!(old_value, 3);

        {
            let mem = cpu.memory.borrow();
            assert_eq!(mem.read_memory(0), Some(0));
            assert_eq!(mem.read_memory(1), Some(2));
            assert_eq!(mem.read_memory(2), Some(1));
            assert_eq!(mem.read_memory(3), Some(0));
        }

        cpu.set_value_in_address(0x8000 + 4, 16).ok();
        assert_eq!(cpu.registers[4], 16);

        if let CPUError::OverflowAddress(address) = cpu.set_value_in_address(0x9000, 16).expect_err("Overflow must occur") {
            assert_eq!(address, 0x9000);
        }
    }

    #[test]
    fn test_read_register() {
        let mut cpu = CPU::new(Rc::new(RefCell::new(Memory::default())));
        cpu.registers[..3].copy_from_slice(&vec![3, 4, 5]);

        assert_eq!(cpu.read_register(0), Some(3));
        assert_eq!(cpu.read_register(1), Some(4));
        assert_eq!(cpu.read_register(2), Some(5));
        assert_eq!(cpu.read_register(3), Some(0));
        assert_eq!(cpu.read_register(7), Some(0));
        assert_eq!(cpu.read_register(8), None);
        assert_eq!(cpu.read_register(u8::MAX), None);
    }

    #[test]
    fn test_write_register() {
        let mut cpu = CPU::new(Rc::new(RefCell::new(Memory::default())));

        cpu.write_register(4, 1234).ok();
        assert_eq!(cpu.registers[4], 1234);

        if let CPUError::OverflowRegister(number) = cpu.write_register(0x10, 16).expect_err("Overflow must occur") {
            assert_eq!(number, 0x10);
        }
    }

    #[test]
    fn test_stack() {
        let mut cpu = CPU::new(Rc::new(RefCell::new(Memory::default())));

        assert_eq!(cpu.pop(), None);

        cpu.push(1);
        cpu.push(2);
        cpu.push(3);

        assert_eq!(cpu.pop(), Some(3));
        assert_eq!(cpu.pop(), Some(2));
        assert_eq!(cpu.pop(), Some(1));
        assert_eq!(cpu.pop(), None);
    }
}
