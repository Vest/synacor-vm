use std::fs;
use std::io::{ErrorKind};
use crate::mem::{Memory, MemoryError};

pub struct VirtualMachine {
    memory: Memory,
}

#[derive(Debug)]
pub enum VirtualMachineError {
    CannotLoadFile(String),
    GeneralError,
}

impl Default for VirtualMachine {
    fn default() -> Self {
        VirtualMachine {
            memory: Memory::default(),
        }
    }
}

impl VirtualMachine {
    pub fn load_binary<F>(&mut self, fn_get_binary: F) -> Result<(), VirtualMachineError>
        where F: FnOnce() -> Vec<u16> {
        let u16_binary = fn_get_binary();
        self.memory.load_data(&u16_binary)?;

        Ok(())
    }

    pub fn get_binary_from_path(path: &str) -> Result<Vec<u16>, VirtualMachineError> {
        fs::read(path).or_else(|err| {
            match err.kind() {
                ErrorKind::NotFound => Err(VirtualMachineError::CannotLoadFile(String::from(path))),
                _ => Err(VirtualMachineError::GeneralError),
            }
        }).and_then(|binary| {
            Ok(binary_to_memory(&binary))
        })
    }
}


fn binary_to_memory(binary: &[u8]) -> Vec<u16> {
    binary.chunks(2)
        .map(|chunk| {
            let le_bytes = [chunk[0], if chunk.len() == 2 { chunk[1] } else { 0 }];
            u16::from_le_bytes(le_bytes)
        }).collect()
}

impl From<MemoryError> for VirtualMachineError {
    fn from(_: MemoryError) -> Self {
        VirtualMachineError::GeneralError
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_binary_to_memory_default() {
        let input = [0x09, 0x00, 0x00, 0x80, 0x01, 0x80, 0x04, 0x00, 0x13, 0x00, 0x00, 0x80];
        let output = binary_to_memory(&input);

        assert_eq!(output.len(), 6);
        assert_eq!(output, [9, 32768, 32769, 4, 19, 32768]);
    }

    #[test]
    fn test_binary_to_memory_empty() {
        let input = [];
        let output = binary_to_memory(&input);

        assert!(output.is_empty());
    }

    #[test]
    fn test_binary_to_memory_single() {
        let input = [0x09];
        let output = binary_to_memory(&input);

        assert_eq!(output.len(), 1);
        assert_eq!(output, [9]);
    }

    #[test]
    fn test_get_binary_from_path_empty() {
        let binary = VirtualMachine::get_binary_from_path("wrong-wrong.bin")
            .expect_err("The file doesn't actually exist, but we loaded it");
        if let VirtualMachineError::CannotLoadFile(file_name) = binary {
            assert_eq!(file_name, "wrong-wrong.bin");
        }
    }

    #[test]
    fn test_get_binary_from_path_small() -> io::Result<()> {
        let path = "test\\small_sample.bin";

        let binary = VirtualMachine::get_binary_from_path(path)
            .expect("The file must exist");
        assert_eq!(binary.len() as u64, fs::metadata(path)?.len() / 2); // length in u8 divided by 2
        assert_eq!(binary[0], 0x0015);
        assert_eq!(binary[1], 0x0015);
        assert_eq!(binary[2], 0x0013);
        assert_eq!(binary[3], 0x0057);

        Ok(())
    }

    #[test]
    fn test_load_binary_small() {
        let path = "test\\small_sample.bin";

        let mut vm = VirtualMachine::default();
        let result = vm.load_binary(|| {
            vec![0x0015, 0x0015, 0x0013, 0x0057]
        }).expect("The binary should load without errors");

        assert_eq!(vm.memory.read_memory(0), Some(0x0015));
        assert_eq!(vm.memory.read_memory(1), Some(0x0015));
        assert_eq!(vm.memory.read_memory(2), Some(0x0013));
        assert_eq!(vm.memory.read_memory(3), Some(0x0057));
    }

    #[test]
    fn test_load_binary_big() {
        let path = "test\\small_sample.bin";

        let mut vm = VirtualMachine::default();
        let result = vm.load_binary(|| {
            vec![0_u16; 32769]
        }).expect_err("The binary is too large. It should never succeed");
    }
}
