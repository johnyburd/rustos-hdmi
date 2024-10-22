#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(decl_macro)]
#![feature(never_type)]
#![no_std]

pub mod atags;
pub mod common;
pub mod gpio;
pub mod interrupt;
pub mod timer;
pub mod uart;
pub mod fb;
pub mod mbox;
pub mod data;
