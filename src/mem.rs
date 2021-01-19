use std::collections::LinkedList;

const MAX_ADDRESS: usize = 0x8000;
const MAX_REGISTERS: usize = 8;

pub enum ErrorKind {
    Overflow(usize),
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
    pub fn load_data(&mut self, data: &[u16]) -> Result<(), ErrorKind> {
        if data.len() > self.memory.len() {
            return Err(ErrorKind::Overflow(data.len()));
        }

        self.memory[..data.len()].copy_from_slice(data);

        Ok(())
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
        let ErrorKind::Overflow(current_length) = res.unwrap_err();
        assert_eq!(current_length, MAX_ADDRESS + 1);
    }
}
