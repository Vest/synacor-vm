use crate::mem::{Memory, MAX_ADDRESS};
use std::cell::RefCell;
use std::rc::Rc;
use log::trace;

pub const MAX_REGISTERS: usize = 8;

#[derive(Debug)]
pub enum CPUError {
    OverflowAddress(u16),
    OverflowRegister(u8),
    PopFromEmptyStack,
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

    pub fn get_current_address(&self) -> u16 {
        self.current_address
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

    pub fn execute(&mut self) -> Result<bool, CPUError> {
        let op_code = self.get_value_from_address(self.current_address)?;
        let a = self.get_value_from_address(self.current_address + 1);
        let b = self.get_value_from_address(self.current_address + 2);
        let c = self.get_value_from_address(self.current_address + 3);
        /*
                    {
                        let a = self.get_value_from_address(self.current_address + 1)?;
                        let b = self.get_value_from_address(self.current_address + 2)?;
                        let c = self.get_value_from_address(self.current_address + 3)?;

                        println!("  Op: {:#02}, a: {:#06X} / {:#05}, b: {:#06X} / {:#05}, c: {:#06X} / {:#05}",
                                 op_code,
                                 a, self.from_raw_to_u16(a)?,
                                 b, self.from_raw_to_u16(b)?,
                                 c, self.from_raw_to_u16(c)?,
                        );
                    }
        */
        let execution_result = match op_code {
            0 => self.halt(),
            1 => self.set(a?, b?),
            2 => self.push(a?),
            3 => self.pop(a?),
            4 => self.eq(a?, b?, c?),
            5 => self.gt(a?, b?, c?),
            6 => self.jmp(a?),
            7 => self.jt(a?, b?),
            8 => self.jf(a?, b?),
            9 => self.add(a?, b?, c?),
            10 => self.mult(a?, b?, c?),
            11 => self.modulo(a?, b?, c?),
            12 => self.and(a?, b?, c?),
            13 => self.or(a?, b?, c?),
            14 => self.not(a?, b?),
            15 => self.rmem(a?, b?),
            16 => self.wmem(a?, b?),
            17 => self.call(a?),
            18 => self.ret(),
            19 => self.out(a?),
            21 => self.noop(),

            _ => Err(CPUError::UnknownOpCode {
                opcode: op_code,
                address: self.current_address,
            }),
        };

        match execution_result? {
            ExecutionResult::Stop => return Ok(true),
            ExecutionResult::Jump(address) => self.current_address = address,
            ExecutionResult::Next(size) => self.current_address += size,
        };

        Ok(false)
    }

    // halt: 0 - stop execution and terminate the program
    fn halt(&self) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: halt!", self.current_address);

        Ok(ExecutionResult::Stop)
    }

    // set: 1 a b - set register <a> to the value of <b>
    fn set(&mut self, raw_a: u16, raw_b: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: set ({:#06X}, {:#06X})", self.current_address, raw_a, raw_b);

        let b = self.from_raw_to_u16(raw_b)?;
        trace!("          b: {:#06X}", b);
        let _ = self.set_value_in_address(raw_a, b)?;

        Ok(ExecutionResult::Next(3))
    }

    // push: 2 a - push <a> onto the stack
    fn push(&mut self, raw_a: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: push ({:#06X})", self.current_address, raw_a);

        let a = self.from_raw_to_u16(raw_a)?;
        self.stack.push(a);

        Ok(ExecutionResult::Next(2))
    }

    // pop: 3 a - remove the top element from the stack and write it into <a>; empty stack = error
    fn pop(&mut self, raw_a: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: pop ({:#06X})", self.current_address, raw_a);

        if let Some(value) = self.stack.pop() {
            self.set_value_in_address(raw_a, value)?;

            Ok(ExecutionResult::Next(2))
        } else {
            Err(CPUError::PopFromEmptyStack)
        }
    }

    // eq: 4 a b c - set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
    fn eq(&mut self, raw_a: u16, raw_b: u16, raw_c: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: eq ({:#06X}, {:#06X}, {:#06X})", self.current_address, raw_a, raw_b, raw_c);

        let b = self.from_raw_to_u16(raw_b)?;
        let c = self.from_raw_to_u16(raw_c)?;
        trace!("          b: {:#06X}, c: {:#06X}", b, c);

        self.set_value_in_address(raw_a,
                                  if b == c { 1 } else { 0 })?;

        Ok(ExecutionResult::Next(4))
    }

    // gt: 5 a b c - set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
    fn gt(&mut self, raw_a: u16, raw_b: u16, raw_c: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: gt ({:#06X}, {:#06X}, {:#06X})", self.current_address, raw_a, raw_b, raw_c);

        let b = self.from_raw_to_u16(raw_b)?;
        let c = self.from_raw_to_u16(raw_c)?;

        self.set_value_in_address(raw_a,
                                  if b > c { 1 } else { 0 })?;

        Ok(ExecutionResult::Next(4))
    }

    // jmp: 6 a - jump to <a>
    fn jmp(&self, raw_a: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: jmp ({:#06X})", self.current_address, raw_a);

        let a = self.from_raw_to_u16(raw_a)?;
        trace!("          a: {:#06X}", a);

        Ok(ExecutionResult::Jump(a))
    }

    // jt: 7 a b - if <a> is nonzero, jump to <b>
    fn jt(&self, raw_a: u16, raw_b: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: jt ({:#06X}, {:#06X})", self.current_address, raw_a, raw_b);

        let a = self.from_raw_to_u16(raw_a)?;
        let b = self.from_raw_to_u16(raw_b)?;
        trace!("          a: {:#06X}, b: {:#06X}", a, b);

        Ok(if a != 0 {
            ExecutionResult::Jump(b)
        } else {
            ExecutionResult::Next(3)
        })
    }

    // jf: 8 a b - if <a> is zero, jump to <b>
    fn jf(&self, raw_a: u16, raw_b: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: jf ({:#06X}, {:#06X})", self.current_address, raw_a, raw_b);

        let a = self.from_raw_to_u16(raw_a)?;
        let b = self.from_raw_to_u16(raw_b)?;
        trace!("          a: {:#06X}, b: {:#06X}", a, b);

        Ok(if a == 0 {
            ExecutionResult::Jump(b)
        } else {
            ExecutionResult::Next(3)
        })
    }

    // add: 9 a b c - assign into <a> the sum of <b> and <c> (modulo 32768)
    fn add(&mut self, raw_a: u16, raw_b: u16, raw_c: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: add ({:#06X}, {:#06X}, {:#06X})", self.current_address, raw_a, raw_b, raw_c);

        let b = self.from_raw_to_u16(raw_b)?;
        let c = self.from_raw_to_u16(raw_c)?;

        let sum = b.wrapping_add(c) & 0x7FFF;
        trace!("          b: {:#06X}, c: {:#06X}, res: {:#06X}", b, c, sum);

        self.set_value_in_address(raw_a, sum)?;

        Ok(ExecutionResult::Next(4))
    }

    // mult: 10 a b c - store into <a> the product of <b> and <c> (modulo 32768)
    fn mult(&mut self, raw_a: u16, raw_b: u16, raw_c: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: mult ({:#06X}, {:#06X}, {:#06X})", self.current_address, raw_a, raw_b, raw_c);

        let b = self.from_raw_to_u16(raw_b)?;
        let c = self.from_raw_to_u16(raw_c)?;

        let mult = b.wrapping_mul(c) & 0x7FFF;
        trace!("          b: {:#06X}, c: {:#06X}, res: {:#06X}", b, c, mult);

        self.set_value_in_address(raw_a, mult)?;
        Ok(ExecutionResult::Next(4))
    }

    // mod: 11 a b c - store into <a> the remainder of <b> divided by <c>
    fn modulo(&mut self, raw_a: u16, b: u16, c: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: mod ({:#06X}, {:#06X}, {:#06X})", self.current_address, raw_a, b, c);

        let rem = b.wrapping_rem(c);
        trace!("          res: {:#06X}", rem);

        self.set_value_in_address(raw_a, rem)?;
        Ok(ExecutionResult::Next(4))
    }

    // and: 12 a b c - stores into <a> the bitwise and of <b> and <c>
    fn and(&mut self, raw_a: u16, raw_b: u16, raw_c: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: and ({:#06X}, {:#06X}, {:#06X})", self.current_address, raw_a, raw_b, raw_c);

        let b = self.from_raw_to_u16(raw_b)?;
        let c = self.from_raw_to_u16(raw_c)?;
        let and = b & c;
        trace!("          b: {:#06X}, c: {:#06X}, res: {:#06X}", b, c, and);

        self.set_value_in_address(raw_a, and)?;
        Ok(ExecutionResult::Next(4))
    }

    // or: 13 a b c - stores into <a> the bitwise or of <b> and <c>
    fn or(&mut self, raw_a: u16, raw_b: u16, raw_c: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: or ({:#06X}, {:#06X}, {:#06X})", self.current_address, raw_a, raw_b, raw_c);

        let b = self.from_raw_to_u16(raw_b)?;
        let c = self.from_raw_to_u16(raw_c)?;
        let or = b | c;
        trace!("          b: {:#06X}, c: {:#06X}, res: {:#06X}", b, c, or);

        self.set_value_in_address(raw_a, or)?;
        Ok(ExecutionResult::Next(4))
    }

    // not: 14 a b - stores 15-bit bitwise inverse of <b> in <a>
    fn not(&mut self, raw_a: u16, raw_b: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: not ({:#06X}, {:#06X})", self.current_address, raw_a, raw_b);

        let b = self.from_raw_to_u16(raw_b)?;
        let not = !b & 0x7FFF;
        trace!("          b: {:#06X}, res: {:#06X}", b, not);

        self.set_value_in_address(raw_a, not)?;
        Ok(ExecutionResult::Next(3))
    }

    // rmem: 15 a b - read memory at address <b> and write it to <a>
    fn rmem(&mut self, raw_a: u16, raw_b: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: rmem ({:#06X}, {:#06X})", self.current_address, raw_a, raw_b);

        let b = self.from_raw_to_u16(raw_b)?;
        let value = self.get_value_from_address(b)?;

        trace!("          b: {:#06X}, res: {:#06X}", b, value);

        self.set_value_in_address(raw_a, value)?;

        Ok(ExecutionResult::Next(3))
    }

    // wmem: 16 a b - write the value from <b> into memory at address <a>
    fn wmem(&mut self, raw_a: u16, raw_b: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: wmem ({:#06X}, {:#06X})", self.current_address, raw_a, raw_b);

        let a = self.from_raw_to_u16(raw_a)?;
        let b = self.from_raw_to_u16(raw_b)?;

        trace!("          a: {:#06X}, b: {:#06X}", a, b);

        self.set_value_in_address(a, b)?;

        Ok(ExecutionResult::Next(3))
    }

    // call: 17 a - write the address of the next instruction to the stack and jump to <a>
    fn call(&mut self, raw_a: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: call ({:#06X})", self.current_address, raw_a);

        let a = self.from_raw_to_u16(raw_a)?;

        self.stack.push(self.current_address + 2);
        Ok(ExecutionResult::Jump(a))
    }

    // ret: 18 - remove the top element from the stack and jump to it; empty stack = halt
    fn ret(&mut self) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: ret", self.current_address);

        if let Some(a) = self.stack.pop() {
            Ok(ExecutionResult::Jump(a))
        } else {
            Ok(ExecutionResult::Stop)
        }
    }

    // out: 19 a - write the character represented by ascii code <a> to the terminal
    fn out(&self, a: u16) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: out ({:#06X})", self.current_address, a);

        let a = (a as u8) as char;

        print!("{}", a);

        Ok(ExecutionResult::Next(2))
    }

    // noop: 21 - no operation
    fn noop(&self) -> Result<ExecutionResult, CPUError> {
        trace!("{:#06X}: noop", self.current_address);

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
    /*
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
     */
}
