pub const MAX_ADDRESS: usize = 0x8000;

pub enum MemoryError {
    DataIsTooLarge(usize),
    OverflowAddress(u16),
}

pub struct Memory {
    memory: [u16; MAX_ADDRESS],
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            memory: [0; MAX_ADDRESS],
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
    fn test_set_value() {
        let mut mem = Memory::default();
        mem.load_data(&[3, 2, 1]).ok();
        let old_value = mem.set_value(0, 0).unwrap_or(u16::MAX);

        assert_eq!(old_value, 3);
        assert_eq!(mem.memory[0], 0);
        assert_eq!(mem.memory[1], 2);
        assert_eq!(mem.memory[2], 1);
        assert_eq!(mem.memory[3], 0);

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
    fn test_write_memory() {
        let mut mem = Memory::default();

        mem.write_memory(0x0123, 1234).ok();
        assert_eq!(mem.memory[0x0123], 1234);

        if let MemoryError::OverflowAddress(address) = mem.write_memory(0x9000, 16).expect_err("Overflow must occur") {
            assert_eq!(address, 0x9000);
        }
    }
}
