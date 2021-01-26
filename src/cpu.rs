use crate::mem::{Memory, MAX_ADDRESS};
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::LinkedList;

const MAX_REGISTERS: usize = 8;

pub enum CPUError {
    OverflowAddress(u16),
    OverflowRegister(u8),
}

enum ExecutionResult {
    Stop,
    Jump(u16),
    Next(u16),
}

pub struct CPU {
    memory: Rc<RefCell<Memory>>,
    registers: [u16; MAX_REGISTERS],
    stack: LinkedList<u16>,

    current_address: u16,
}

impl CPU {
    pub fn new(mem: Rc<RefCell<Memory>>) -> CPU {
        CPU {
            memory: mem,
            registers: [0; MAX_REGISTERS],
            stack: LinkedList::new(),

            current_address: 0,
        }
    }

    pub fn get_value(&self, address: u16) -> Result<u16, CPUError> {
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

    pub fn set_value(&mut self, address: u16, value: u16) -> Result<u16, CPUError> {
        match address {
            0..=0x7FFF => {
                self.memory.borrow_mut()
                    .write_memory(address, value)
                    .or(Err(CPUError::OverflowAddress(address)))
            }
            0x8000..=0x8007 => {
                let reg_num = get_registry_from_address(address).unwrap();
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

    pub fn push(&mut self, value: u16) {
        self.stack.push_back(value);
    }

    pub fn pop(&mut self) -> Option<u16> {
        self.stack.pop_back()
    }

    pub fn execute(&mut self) -> Result<(), CPUError> {
        loop {
            let (op_code, a, b, c) = {
                let op_code = self.get_value(self.current_address)?;
                let mem_a = self.get_value(self.current_address + 1);
                let mem_b = self.get_value(self.current_address + 2);
                let mem_c = self.get_value(self.current_address + 3);

                (op_code, mem_a, mem_b, mem_c)
            };

            let execution_result = match op_code {
                0 => self.halt(),
                19 => self.out(a?),
                21 => self.noop(),

                _ => ExecutionResult::Stop,
            };

            match execution_result {
                ExecutionResult::Stop => break,
                ExecutionResult::Jump(address) => self.current_address = address,
                ExecutionResult::Next(size) => self.current_address += size,
            }
        }

        Ok(())
    }

    // halt: 0 - stop execution and terminate the program
    fn halt(&self) -> ExecutionResult {
        ExecutionResult::Stop
    }

    // out: 19 a - write the character represented by ascii code <a> to the terminal
    fn out(&self, a: u16) -> ExecutionResult {
        let a = (a as u8) as char;

        print!("{}", a);

        ExecutionResult::Next(2)
    }

    // noop: 21 - no operation
    fn noop(&self) -> ExecutionResult {
        ExecutionResult::Next(1)
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
        let old_value = cpu.set_value(0, 0).unwrap_or(u16::MAX);

        assert_eq!(old_value, 3);

        {
            let mem = cpu.memory.borrow();
            assert_eq!(mem.read_memory(0), Some(0));
            assert_eq!(mem.read_memory(1), Some(2));
            assert_eq!(mem.read_memory(2), Some(1));
            assert_eq!(mem.read_memory(3), Some(0));
        }

        cpu.set_value(0x8000 + 4, 16).ok();
        assert_eq!(cpu.registers[4], 16);

        if let CPUError::OverflowAddress(address) = cpu.set_value(0x9000, 16).expect_err("Overflow must occur") {
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
