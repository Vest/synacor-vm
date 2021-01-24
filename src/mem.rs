use std::collections::LinkedList;

const MAX_ADDRESS: usize = 0x8000;
const MAX_REGISTERS: usize = 8;

pub enum MemoryError {
    DataIsTooLarge(usize),
    OverflowAddress(u16),
    OverflowRegister(u8),
}

pub struct Memory {
    memory: [u16; MAX_ADDRESS],
    registers: [u16; MAX_REGISTERS],
    stack: LinkedList<u16>,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            memory: [0; MAX_ADDRESS],
            registers: [0; MAX_REGISTERS],
            stack: LinkedList::new(),
        }
    }
}

impl Memory {
    pub fn load_data(&mut self, data: &[u16]) -> Result<(), MemoryError> {
        if data.len() > self.memory.len() {
            return Err(MemoryError::DataIsTooLarge(data.len()));
        }

        self.memory[..data.len()].copy_from_slice(data);

        Ok(())
    }

    pub fn set_value(&mut self, address: u16, value: u16) -> Result<u16, MemoryError> {
        match address {
            0..=0x7FFF => self.write_memory(address, value),
            0x8000..=0x8007 => {
                let reg_num = get_registry_from_address(address).unwrap();
                self.write_register(reg_num, value)
            }
            _ => Err(MemoryError::OverflowAddress(address)),
        }
    }

    pub fn read_memory(&self, address: u16) -> Option<u16> {
        match address {
            0..=0x7FFF => Some(self.memory[address as usize]),
            _ => None,
        }
    }

    pub fn write_memory(&mut self, address: u16, value: u16) -> Result<u16, MemoryError> {
        match address {
            0..=0x7FFF if address < MAX_ADDRESS as u16 => {
                let old_value = self.memory[address as usize];
                self.memory[address as usize] = value;

                Ok(old_value)
            }
            _ => Err(MemoryError::OverflowAddress(address)),
        }
    }

    pub fn read_register(&self, number: u8) -> Option<u16> {
        match number {
            0..=7 => Some(self.registers[number as usize]),
            _ => None,
        }
    }

    pub fn write_register(&mut self, number: u8, value: u16) -> Result<u16, MemoryError> {
        match number {
            0..=7 => {
                let old_value = self.registers[number as usize];
                self.registers[number as usize] = value;

                Ok(old_value)
            }
            _ => Err(MemoryError::OverflowRegister(number)),
        }
    }

    pub fn push(&mut self, value: u16) {
        self.stack.push_back(value);
    }

    pub fn pop(&mut self) -> Option<u16> {
        self.stack.pop_back()
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
    fn test_load_data_ok() {
        let mut mem = Memory::default();
        let res = mem.load_data(&[0, 1, 2, 3]);

        assert_eq!(mem.memory[0..4], [0, 1, 2, 3]);
        assert!(res.is_ok());
    }

    #[test]
    fn test_load_data_overflow() {
        let mut mem = Memory::default();
        let large_block = vec![1; MAX_ADDRESS + 1];

        let res = mem.load_data(&large_block);

        assert_eq!(mem.memory[0..4], [0, 0, 0, 0]);
        assert!(res.is_err());

        if let MemoryError::DataIsTooLarge(current_length) = res.expect_err("Overflow must occur") {
            assert_eq!(current_length, MAX_ADDRESS + 1);
        }
    }

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
        let old_value = mem.set_value(0, 0).unwrap_or(u16::MAX);

        assert_eq!(old_value, 3);
        assert_eq!(mem.memory[0], 0);
        assert_eq!(mem.memory[1], 2);
        assert_eq!(mem.memory[2], 1);
        assert_eq!(mem.memory[3], 0);

        mem.set_value(0x8000 + 4, 16).ok();
        assert_eq!(mem.registers[4], 16);

        if let MemoryError::OverflowAddress(address) = mem.set_value(0x9000, 16).expect_err("Overflow must occur") {
            assert_eq!(address, 0x9000);
        }
    }

    #[test]
    fn test_read_memory() {
        let mut mem = Memory::default();
        mem.load_data(&[3, 2, 1]).ok();

        assert_eq!(mem.read_memory(0), Some(3));
        assert_eq!(mem.read_memory(1), Some(2));
        assert_eq!(mem.read_memory(2), Some(1));
        assert_eq!(mem.read_memory(3), Some(0));
        assert_eq!(mem.read_memory(0x8000), None);
        assert_eq!(mem.read_memory(MAX_ADDRESS as u16), None);
        assert_eq!(mem.read_memory(u16::MAX), None);
    }

    #[test]
    fn test_read_register() {
        let mut mem = Memory::default();
        mem.registers[..3].copy_from_slice(&vec![3, 4, 5]);

        assert_eq!(mem.read_register(0), Some(3));
        assert_eq!(mem.read_register(1), Some(4));
        assert_eq!(mem.read_register(2), Some(5));
        assert_eq!(mem.read_register(3), Some(0));
        assert_eq!(mem.read_register(7), Some(0));
        assert_eq!(mem.read_register(8), None);
        assert_eq!(mem.read_register(u8::MAX), None);
    }

    #[test]
    fn test_stack() {
        let mut mem = Memory::default();

        assert_eq!(mem.pop(), None);

        mem.push(1);
        mem.push(2);
        mem.push(3);

        assert_eq!(mem.pop(), Some(3));
        assert_eq!(mem.pop(), Some(2));
        assert_eq!(mem.pop(), Some(1));
        assert_eq!(mem.pop(), None);
    }

    #[test]
    fn test_write_memory() {
        let mut mem = Memory::default();

        mem.write_memory(0x0123, 1234).ok();
        assert_eq!(mem.memory[0x0123], 1234);

        if let MemoryError::OverflowAddress(address) = mem.write_memory(0x9000, 16).expect_err("Overflow must occur") {
            assert_eq!(address, 0x9000);
        }
    }

    #[test]
    fn test_write_register() {
        let mut mem = Memory::default();

        mem.write_register(4, 1234).ok();
        assert_eq!(mem.registers[4], 1234);

        if let MemoryError::OverflowRegister(number) = mem.write_register(0x10, 16).expect_err("Overflow must occur") {
            assert_eq!(number, 0x10);
        }
    }
}
