use std::fs;
use std::io::{ErrorKind};
use crate::mem::{Memory, MemoryError};
use crate::cpu::{CPU};
use std::cell::RefCell;
use std::rc::Rc;
use std::iter::FromIterator;

pub struct VirtualMachine {
    memory: Rc<RefCell<Memory>>,
    pub cpu: CPU,
}

#[derive(Debug)]
pub enum VirtualMachineError {
    CannotLoadFile(String),
    GeneralError,
}

impl Default for VirtualMachine {
    fn default() -> Self {
        let mem = Rc::new(RefCell::new(Memory::default()));
        VirtualMachine {
            memory: Rc::clone(&mem),
            cpu: CPU::new(Rc::clone(&mem)),
        }
    }
}

impl VirtualMachine {
    pub fn load_binary<F>(&mut self, fn_get_binary: F) -> Result<(), VirtualMachineError>
        where F: FnOnce() -> Vec<u16> {
        let u16_binary = fn_get_binary();
        self.memory.borrow_mut()
            .load_data(&u16_binary)?;

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

    pub fn next_step(&mut self) -> Result<bool, VirtualMachineError> {
        match self.cpu.execute() {
            Ok(to_stop) => Ok(to_stop),
            Err(_) => Err(VirtualMachineError::GeneralError)
        }
    }

    pub fn run(&mut self) {
        while let Ok(to_stop) = self.next_step() {
            if to_stop {
                break;
            }
        }
    }

    pub fn dump_registry(&self) {
        println!(r#"--- Registers ---
{}
{}
"#,
                 String::from_iter((0..crate::cpu::MAX_REGISTERS as u8)
                     .map(|reg_num| format!("{:#6} ", reg_num))),
                 String::from_iter(
                     (0..crate::cpu::MAX_REGISTERS as u8).
                         filter_map(|reg_num| self.cpu.read_register(reg_num))
                         .map(|reg_value| format!("{:#06X} ", reg_value))
                 ));
    }

    pub fn get_current_address(&self) -> u16 {
        self.cpu.get_current_address()
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
    use std::path::PathBuf;

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
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test/small_sample.bin");
        let path = path.to_str().unwrap();

        let binary = VirtualMachine::get_binary_from_path(path)
            .expect(format!("The file must exist: {}", path).as_str());
        assert_eq!(binary.len() as u64, fs::metadata(path)?.len() / 2); // length in u8 divided by 2
        assert_eq!(binary[0], 0x0015);
        assert_eq!(binary[1], 0x0015);
        assert_eq!(binary[2], 0x0013);
        assert_eq!(binary[3], 0x0057);

        Ok(())
    }

    #[test]
    fn test_load_binary_small() {
        let mut vm = VirtualMachine::default();
        let _ = vm.load_binary(|| {
            vec![0x0015, 0x0015, 0x0013, 0x0057]
        }).expect("The binary should load without errors");

        let vm_memory = vm.memory.borrow();
        assert_eq!(vm_memory.read_memory(0), Some(0x0015));
        assert_eq!(vm_memory.read_memory(1), Some(0x0015));
        assert_eq!(vm_memory.read_memory(2), Some(0x0013));
        assert_eq!(vm_memory.read_memory(3), Some(0x0057));
    }

    #[test]
    fn test_load_binary_big() {
        let mut vm = VirtualMachine::default();

        vm.load_binary(|| {
            vec![0_u16; 32769]
        }).expect_err("The binary is too large. It should never succeed");
    }
}
