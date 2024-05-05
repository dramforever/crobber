use std::marker::PhantomData;

#[cfg(all(unix, target_arch = "x86_64"))]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct RawCrob<'a> {
    sp: *const usize,
    marker: PhantomData<&'a mut [usize]>,
}

#[cfg(all(unix, target_arch = "riscv64"))]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct RawCrob<'a> {
    sp: *const usize,
    ra: usize,
    marker: PhantomData<&'a mut [usize]>,
}

#[cfg(all(unix, target_arch = "x86_64"))]
const STACK_ALIGN: usize = 16;
#[cfg(all(unix, target_arch = "riscv64"))]
const STACK_ALIGN: usize = 16;

#[cfg(all(unix, target_arch = "x86_64"))]
fn crob_yield(crob: RawCrob, data: usize) -> (RawCrob, usize) {
    use std::arch::asm;

    let res_crob: *const usize;
    let res_data;

    unsafe {
        asm!(
            "push rbx",
            "push rbp",
            "lea rcx, [2f + rip]",
            "push rcx",
            "xchg rsp, rdi",
            "ret",
            "2:",
            "pop rbp",
            "pop rbx",
            inout("rdi") crob.sp => res_crob,
            inout("rsi") data => res_data,
            out("r12") _,
            out("r13") _,
            out("r14") _,
            out("r15") _,
            clobber_abi("sysv64")
        );
    };

    (
        RawCrob {
            sp: res_crob,
            marker: PhantomData,
        },
        res_data,
    )
}

#[cfg(all(unix, target_arch = "riscv64"))]
fn crob_yield(crob: RawCrob, data: usize) -> (RawCrob, usize) {
    use std::arch::asm;

    let res_crob: *const usize;
    let res_ra: usize;
    let res_data;

    #[cfg(all(unix, target_arch = "riscv64"))]
    unsafe {
        asm!(
            "addi sp, sp, -16",
            "sd s0, (sp)",
            "sd s1, 8(sp)",
            "mv a0, sp",
            "mv sp, {cr}",
            "jalr ra, 0(t0)",
            "ld s0, (sp)",
            "ld s1, 8(sp)",
            "addi sp, sp, 16",
            cr = in(reg) crob.sp,
            out("a0") res_crob,
            in("t0") crob.ra,
            out("ra") res_ra,
            inout("a2") data => res_data,
            out("s2") _,
            out("s3") _,
            out("s4") _,
            out("s5") _,
            out("s6") _,
            out("s7") _,
            out("s8") _,
            out("s9") _,
            out("s10") _,
            out("s11") _,
            out("fs0") _,
            out("fs1") _,
            out("fs2") _,
            out("fs3") _,
            out("fs4") _,
            out("fs5") _,
            out("fs6") _,
            out("fs7") _,
            out("fs8") _,
            out("fs9") _,
            out("fs10") _,
            out("fs11") _,
            clobber_abi("C")
        );
    };

    (
        RawCrob {
            sp: res_crob,
            ra: res_ra,
            marker: PhantomData,
        },
        res_data,
    )
}

#[cfg(all(unix, target_arch = "riscv64"))]
std::arch::global_asm!(
    ".pushsection .text._crobber_shim, \"ax\"",
    "_crobber_shim:",
    "mv a1, ra",
    "ld t1, (sp)",
    "addi sp, sp, 16",
    "jr t1",
    ".popsection",
);

impl<'a> RawCrob<'a> {
    #[cfg(all(unix, target_arch = "x86_64"))]
    pub fn new(stack: &mut [usize], start: extern "C" fn(RawCrob, usize) -> !) -> Self {
        assert!(stack.len() > 8, "Stack too small");

        let mut ptr = stack.as_mut_ptr_range().end;

        unsafe {
            let off = ptr.align_offset(STACK_ALIGN);
            ptr = ptr.sub(2 - off);
            *ptr = start as usize;
        }

        Self {
            sp: ptr,
            marker: PhantomData,
        }
    }

    #[cfg(all(unix, target_arch = "riscv64"))]
    pub fn new(stack: &mut [usize], start: extern "C" fn(RawCrob, usize) -> !) -> Self {
        assert!(stack.len() > 8, "Stack too small");

        let mut ptr = stack.as_mut_ptr_range().end;

        unsafe {
            let off = ptr.align_offset(STACK_ALIGN);
            ptr = ptr.sub(4 - off);
            *ptr = start as _;
        }

        extern "C" {
            fn _crobber_shim() -> !;
        }

        Self {
            sp: ptr,
            ra: _crobber_shim as _,
            marker: PhantomData,
        }
    }

    pub fn call(&mut self, data: usize) -> usize {
        let (cr, data) = crob_yield(*self, data);
        *self = cr;
        data
    }
}
