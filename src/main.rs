use crate::vm::{VirtualMachine};
use std::io::{self, Write};
use std::process::exit;

mod vm;
mod mem;
mod cpu;

fn main() {
    env_logger::builder()
        .format_timestamp(None)
        .init();

    println!("Let's start the VM!!!!");

    let mut vm = vm::VirtualMachine::default();
    vm.load_binary(|| {
        VirtualMachine::get_binary_from_path("challenge.bin").unwrap_or_else(|err| {
            eprintln!("Couldn't load the test file: {:?}", err);
            Vec::new()
        })
    }).expect("The file 'challenge.bin' couldn't be loaded");

    println!("Type 'exit' to hm... exit");
    let mut buffer = String::new();
    while let Ok(_) = io::stdin().read_line(&mut buffer) {
        match buffer.trim_end() {
            "exit" => break,
            "regs" => vm.dump_registry(),
            "where" => {
                io::stdout().flush().unwrap();
                println!("\n{0:#6} / {0:#06X}", vm.get_current_address());
            }
            "run" => vm.run(),
            buf @ _ => {
                if buf.starts_with("until ") {
                    if let Ok(pos) = u16::from_str_radix(buf.trim_start_matches("until 0x"), 16) {
                        vm.run_until(pos);
                    } else if let Ok(pos) = u16::from_str_radix(buf.trim_start_matches("until "), 10) {
                        vm.run_until(pos);
                    } else {
                        eprintln!("Couldn't parse the command: {}", buf);
                    }
                } else {
                    match vm.next_step() {
                        Ok(to_stop) if to_stop => break,
                        Err(err) => {
                            eprintln!("Unexpected error: {:?}\n", err);
                            exit(-1);
                        }
                        _ => {}
                    }
                }
            }
        }

        buffer.clear();
    }
}
