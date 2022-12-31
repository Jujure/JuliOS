use crate::println;
use crate::utils::mutex::AsyncMutex;

use super::scheduler::SCHEDULER;

use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

use alloc::alloc::{alloc, dealloc, Layout};
use lazy_static::lazy_static;

const STACK_SIZE: usize = 4096 * 20;

lazy_static! {
    pub static ref RUNNING_THREAD: AsyncMutex<ThreadId> = AsyncMutex::new(ThreadId(0));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadId(pub u64);

impl ThreadId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        ThreadId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub fn exit() {
    println!("Exiting thread");
    let thread: *mut Thread;
    {
        let mut scheduler = SCHEDULER.try_lock().unwrap();
        thread = scheduler
            .get_thread(ThreadId(0))
            .unwrap()
            .as_ptr();
    } // Drop scheduler mutex guard

    unsafe {
        (&mut* thread).run();
    }
}

pub struct Thread {
    pub id: ThreadId,
    pub entry_point: u64,
    pub started: bool,
    pub rsp: u64,
}

impl Thread {
    pub fn new(entry_point: u64) -> Self {
        unsafe {
            Thread {
                id: ThreadId::new(),
                entry_point: entry_point,
                started: false,
                rsp: alloc(Layout::new::<[u8; STACK_SIZE]>()) as u64 + STACK_SIZE as u64,
            }
        }
    }

    pub fn run(&mut self) {
        println!("Running thread {:?}", self.id);
        unsafe {
            let mut current_thread_guard = RUNNING_THREAD.try_lock().unwrap();
            let current_rsp: u64;
            asm!(
                "push rsp",    // Recover current rsp
                "pop {out}",
                "sub {out}, 56", // Offset to saved registers
                out = out(reg) current_rsp, // Save thread rsp
            );

            let mut scheduler = SCHEDULER.try_lock().unwrap();
            let current_thread = scheduler.get_thread(*current_thread_guard).unwrap();
            current_thread.borrow_mut().rsp = current_rsp;

            *current_thread_guard = self.id; // change running thread
        } // The scheduler and running thread guards is dropped here

        unsafe {
            if self.started {
                asm!(
                    "push rax",       // Save current thread regs
                    "push rbx",
                    "push rcx",
                    "push rdx",
                    "push rbp",
                    "push rsi",
                    "push rdi",

                    "push {rsp}", // Set stack pointer to the new thread
                    "pop rsp",

                    "pop rdi",       // Restore new thread regs
                    "pop rsi",
                    "pop rbp",
                    "pop rdx",
                    "pop rcx",
                    "pop rbx",
                    "pop rax",
                    rsp = in(reg) self.rsp,
                );
            } else {
                self.started = true;
                asm!(
                    "push rax",       // Save current thread regs
                    "push rbx",
                    "push rcx",
                    "push rdx",
                    "push rbp",
                    "push rsi",
                    "push rdi",

                    "push {rsp}",    // Set stack pointer to the new thread
                    "pop rsp",
                    "jmp {rip}",     // Jump to thread routine
                    rsp = in(reg) self.rsp,
                    rip = in(reg) self.entry_point,
                );
            }
        }
    }
}
