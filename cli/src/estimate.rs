//! Bare bones upper bound estimator that uses the rv32im
//! emulation utils for fast lookups in the opcode list
//! to extract the cycle count from an elf.

use anyhow::Result;
use indicatif::ProgressIterator;
use risc0_binfmt::{MemoryImage, Program};
use risc0_circuit_rv32im::prove::emu::{
    exec::{execute, DEFAULT_SEGMENT_LIMIT_PO2},
    testutil::DEFAULT_SESSION_LIMIT,
};
use risc0_zkvm::GUEST_MAX_MEM;
use risc0_zkvm_platform::PAGE_SIZE;

use self::emu_syscall::BasicSyscall;

pub fn estimate<E: MkImage>(elf: E, max_cycles: Option<u64>) -> Result<()> {
    let cycles: usize = get_cycle_count(elf, max_cycles)?;
    println!("number of cycles: {cycles}");

    Ok(())
}

/// Get the total number of cycles by stepping through the ELF using emulation
/// tools from the risc0_circuit_rv32im module.
pub fn get_cycle_count<E: MkImage>(elf: E, max_cycles: Option<u64>) -> Result<usize> {
    execute(
        elf.mk_image()?,
        DEFAULT_SEGMENT_LIMIT_PO2,
        max_cycles.or(DEFAULT_SESSION_LIMIT),
        &BasicSyscall::default(),
        None,
    )?
    .segments
    .iter()
    .progress()
    .try_fold(0, |acc, s| -> Result<usize> {
        let trace = s.preflight()?;
        let segment_cycles = trace.pre.cycles.len() + trace.body.cycles.len();
        Ok(acc + segment_cycles)
    })
}

/// Helper trait for loading an image from an elf.
pub trait MkImage {
    fn mk_image(self) -> Result<MemoryImage>;
}
impl<'a> MkImage for &'a [u8] {
    fn mk_image(self) -> Result<MemoryImage> {
        let program = Program::load_elf(self, GUEST_MAX_MEM as u32)?;
        MemoryImage::new(&program, PAGE_SIZE as u32)
    }
}

pub mod emu_syscall {
    //! The following is copied from risc0 emu test utils, likely this is okay for our use case since we only want the cycle count.
    //! https://github.com/anagrambuild/risc0/blob/eb331d7ee30bc9ccf944bb1ea4835e60e21c25a2/risc0/circuit/rv32im/src/prove/emu/exec/tests.rs#L41

    use std::cell::RefCell;

    use anyhow::Result;
    use risc0_circuit_rv32im::prove::emu::{
        addr::ByteAddr,
        exec::{Syscall, SyscallContext},
    };
    use risc0_zkvm_platform::syscall::reg_abi::{REG_A4, REG_A5};

    #[derive(Default, Clone)]
    pub struct BasicSyscallState {
        syscall: String,
        from_guest: Vec<u8>,
        into_guest: Vec<u8>,
    }

    #[derive(Default)]
    pub struct BasicSyscall {
        state: RefCell<BasicSyscallState>,
    }

    impl Syscall for BasicSyscall {
        fn syscall(
            &self,
            syscall: &str,
            ctx: &mut dyn SyscallContext,
            guest_buf: &mut [u32],
        ) -> Result<(u32, u32)> {
            self.state.borrow_mut().syscall = syscall.to_string();
            let buf_ptr = ByteAddr(ctx.peek_register(REG_A4)?);
            let buf_len = ctx.peek_register(REG_A5)?;
            self.state.borrow_mut().from_guest = ctx.peek_region(buf_ptr, buf_len)?;
            let guest_buf_bytes: &mut [u8] = bytemuck::cast_slice_mut(guest_buf);
            let into_guest = &self.state.borrow().into_guest;
            guest_buf_bytes[..into_guest.len()].clone_from_slice(into_guest);
            Ok((0, 0))
        }
    }
}

#[cfg(test)]
mod estimate_tests {
    use anyhow::Result;
    use risc0_binfmt::MemoryImage;
    use risc0_circuit_rv32im::prove::emu::testutil::basic as basic_test_program;
    use risc0_zkvm::PAGE_SIZE;

    use super::MkImage;
    use crate::estimate;

    impl MkImage for MemoryImage {
        fn mk_image(self) -> Result<MemoryImage> {
            Ok(self)
        }
    }

    #[test]
    fn estimate_basic() {
        let program = basic_test_program();
        let image = MemoryImage::new(&program, PAGE_SIZE as u32)
            .expect("failed to create image from basic program");
        let res = estimate::get_cycle_count(image, None);

        assert_eq!(res.ok(), Some(15790));
    }
}
