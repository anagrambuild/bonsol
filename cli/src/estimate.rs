//! Bare bones upper bound estimator that uses the rv32im
//! emulation utils for fast lookups in the opcode list
//! to extract the cycle count from an elf.

use anyhow::Result;
use risc0_binfmt::{MemoryImage, Program};
use risc0_zkvm::{ExecutorEnv, ExecutorImpl, GUEST_MAX_MEM};
use risc0_zkvm_platform::PAGE_SIZE;

pub fn estimate<E: MkImage>(elf: E, env: ExecutorEnv) -> Result<()> {
    let cycles = get_cycle_count(elf, env)?;
    println!("total: {cycles}");

    Ok(())
}

/// Get the total number of cycles by stepping through the ELF using emulation
/// tools from the risc0_circuit_rv32im module.
pub fn get_cycle_count<E: MkImage>(elf: E, env: ExecutorEnv) -> Result<u64> {
    Ok(ExecutorImpl::new(env, elf.mk_image()?)?.run()?.total_cycles)
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

#[cfg(test)]
mod estimate_tests {
    use anyhow::Result;
    use risc0_binfmt::MemoryImage;
    use risc0_circuit_rv32im::prove::emu::{
        exec::DEFAULT_SEGMENT_LIMIT_PO2,
        testutil::{basic as basic_test_program, DEFAULT_SESSION_LIMIT},
    };
    use risc0_zkvm::{ExecutorEnv, PAGE_SIZE};

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
        let mut env = &mut ExecutorEnv::builder();
        env = env
            .segment_limit_po2(DEFAULT_SEGMENT_LIMIT_PO2 as u32)
            .session_limit(DEFAULT_SESSION_LIMIT);
        let image = MemoryImage::new(&program, PAGE_SIZE as u32)
            .expect("failed to create image from basic program");
        let res = estimate::get_cycle_count(image, env.build().unwrap());

        assert_eq!(res.ok(), Some(16384));
    }
}
