use crate::println;
use crate::utils::mutex::AsyncMutex;

use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

use alloc::alloc::{alloc, dealloc, Layout};
use lazy_static::lazy_static;

const STACK_SIZE: usize = 4096 * 20;

lazy_static! {
    pub static ref RUNNING_THREAD: AsyncMutex<ThreadId> = AsyncMutex::new(ThreadId(0));
    pub static ref KERNEL_THREAD: AsyncMutex<Thread> = {
        let k_rsp: u64;
        unsafe {
            asm!(
                "push rsp",    // Recover current rsp
                "pop {out}",
                out = out(reg) k_rsp, // Save current rsp
            );
        }
        let thread: Thread = Thread {
            id: ThreadId(0),
            rsp: k_rsp,
        };
        AsyncMutex::new(thread)
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadId(u64);

impl ThreadId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        ThreadId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub fn exit() {
    println!("Exiting");
    KERNEL_THREAD.try_lock().unwrap().run();
}

pub struct Thread {
    pub id: ThreadId,
    rsp: u64
}

impl Thread {
    pub fn new() -> Self {
        unsafe {
            Thread {
                id: ThreadId::new(),
                rsp: alloc(Layout::new::<[u8; STACK_SIZE]>()) as u64,
            }
        }
    }

    pub async fn start(&mut self, rip: u64) {
        unsafe {
            asm!(
                "pusha",       // Save current thread regs
                "push rsp",    // Recover current rsp
                "pop {out}",
                out = out(reg) self.rsp, // Save current rsp
            );
        }

        *RUNNING_THREAD.lock().await = self.id;
        unsafe {
            asm!(
                "push {rsp}",
                "pop rsp",
                "jmp {rip}",
                rsp = in(reg) self.rsp,
                rip = in(reg) rip,
            );
        }
    }

    pub async fn run(&mut self) {
        unsafe {
            asm!(
                "pusha",       // Save current thread regs
                "push rsp",    // Recover current rsp
                "pop {out}",
                out = out(reg) self.rsp, // Save current rsp
            );

            *RUNNING_THREAD.lock().await = self.id; // change running thread

            asm!(
                "push {rsp}", // Set stack pointer to the new thread
                "pop rsp",
                "popa",       // Restore new thread regs
                rsp = in(reg) self.rsp,
            );
        }
    }
}