#![feature(llvm_asm)]

const SSIZE: isize = 48;

#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    rsp: u64,
}

fn hello() -> ! {
    println!("I LOVE WAKING UP ON A NEW STACK!");
    loop {}
}

unsafe fn gt_switch(new: *const ThreadContext) {
    llvm_asm!(
    "
        mov 0x00($0), %rsp
        ret
        "
    :
    : "r"(new)
    :
    : "alignstack" // it will work without this now, will need it later
    );
}

fn main() {
    let mut ctx = ThreadContext::default();
    let mut stack = vec![0_u8; SSIZE as usize];
    unsafe {
        let stack_bottom = stack.as_mut_ptr().offset(SSIZE);
        let sb_aligned = (stack_bottom as usize & !15) as *mut u8;
        std::ptr::write(sb_aligned.offset(-16) as *mut u64, hello as u64);
        ctx.rsp = sb_aligned.offset(-16) as u64;

        let stack_ptr = stack.as_mut_ptr();
        for i in (0..SSIZE).rev() {
            println!(
                "mem {}, val: {}",
                stack_ptr.offset(i as isize) as usize,
                *stack_ptr.offset(i as isize)
            )
        }
        gt_switch(&mut ctx);
    }
}
