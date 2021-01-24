use crate::vm::{VirtualMachine};

mod vm;
mod mem;

fn main() {
    println!("Hello, world!");
    let mut vm = vm::VirtualMachine::default();
    vm.load_binary(|| {
        VirtualMachine::get_binary_from_path("test").unwrap_or_else(|err| {
            eprintln!("Couldn't load the test file: {:?}", err);
            Vec::new()
        })
    });
}
