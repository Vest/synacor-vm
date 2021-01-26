use crate::vm::{VirtualMachine};

mod vm;
mod mem;
mod cpu;

fn main() {
    println!("Let's start the VM!!!!");

    let mut vm = vm::VirtualMachine::default();
    vm.load_binary(|| {
        VirtualMachine::get_binary_from_path("challenge.bin").unwrap_or_else(|err| {
            eprintln!("Couldn't load the test file: {:?}", err);
            Vec::new()
        })
    }).expect("The file 'challenge.bin' couldn't be loaded");

    vm.cpu.execute()
        .unwrap_or_else(|err| {
            eprintln!("Unexpected error: {:?}\n", err);

            vm.cpu.dump_cpu();
        })
}
