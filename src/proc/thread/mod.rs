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
    let k_thread: *mut Thread;
    {
        let mut scheduler = SCHEDULER.try_lock().unwrap();
        k_thread = scheduler
            .get_thread(ThreadId(0))
            .unwrap()
            .as_ptr();
            scheduler.exit(*RUNNING_THREAD.try_lock().unwrap());
    } // Drop scheduler mutex guard

    unsafe {
        (&mut* k_thread).run();
    }
}

pub fn routine() {
    println!("Routine executed");
    exit();
}

pub struct Thread {
    pub id: ThreadId,
    pub entry_point: u64,
    pub started: bool,
    pub rsp: u64,
    pub base_stack: u64
}

impl Thread {
    pub fn new(entry_point: u64) -> Self {
        unsafe {
            let stack_bottom = alloc(Layout::new::<[u8; STACK_SIZE]>()) as u64;
            Thread {
                id: ThreadId::new(),
                entry_point: entry_point,
                started: false,
                rsp: stack_bottom + STACK_SIZE as u64,
                base_stack: stack_bottom,
            }
        }
    }

    pub fn exit(&self) {
        unsafe {
            dealloc(self.base_stack as *mut u8, Layout::new::<[u8; STACK_SIZE]>());
        }
    }

    pub fn run(&mut self) {
        println!("Running thread {:?}", self.id);
        unsafe {
            let mut current_thread_guard = RUNNING_THREAD.try_lock().unwrap();

            let mut scheduler = SCHEDULER.try_lock().unwrap();
            if let Some(current_thread) = scheduler.get_thread(*current_thread_guard) {
                let current_rsp: u64;
                asm!(
                    "push rsp",    // Recover current rsp
                    "pop {out}",
                    "sub {out}, 56", // Offset to saved registers
                    out = out(reg) current_rsp, // Save thread rsp
                );
                current_thread.borrow_mut().rsp = current_rsp;
            }
            else { // Thread does not exists anymore
                *current_thread_guard = self.id; // change running thread
                asm!( // Just switch to new thead without saving registers
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
                return;
            }

            *current_thread_guard = self.id; // change running thread
        } // The scheduler and running thread guards are dropped here

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
