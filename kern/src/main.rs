#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(optin_builtin_traits)]
#![feature(ptr_internals)]
#![feature(raw_vec_internals)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

extern crate alloc;

pub mod allocator;
pub mod console;
pub mod fs;
pub mod mutex;
pub mod shell;
pub mod param;
pub mod process;
pub mod traps;
pub mod vm;

use console::{kprintln, kprint};

use allocator::Allocator;
use fs::FileSystem;
use process::GlobalScheduler;
use traps::irq::Irq;
use vm::VMManager;

use pi::atags::Atags;

#[cfg_attr(not(test), global_allocator)]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();
pub static FILESYSTEM: FileSystem = FileSystem::uninitialized();
pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();
pub static VMM: VMManager = VMManager::uninitialized();
pub static IRQ: Irq = Irq::uninitialized();

fn kmain() -> ! {
    use pi::timer::spin_sleep;
    use core::time::Duration;

    spin_sleep(Duration::from_secs(2));
    for atag in Atags::get() {
        kprintln!("{:#?}", atag);
    }

    unsafe {
        ALLOCATOR.initialize();
        FILESYSTEM.initialize();
    }

    /*
    use alloc::vec::Vec;

    let mut v = Vec::new();
    for i in 0..50 {
        v.push(i);
        kprintln!("{:?}", v);
    }*/

    kprintln!("Welcome to cs3210!");
    shell::shell("> ");
}
