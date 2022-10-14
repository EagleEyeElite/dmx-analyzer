extern crate hwloc;

use std::thread;
use std::time::Duration;
use rppal::gpio::{Gpio, Level, Trigger};
use std::io;
use scheduler::set_self_priority;
use libc;
use core::sync::atomic::{Ordering, AtomicBool, AtomicU32};
use scheduler::Which::Process;
use hwloc::Topology;

static LAST: AtomicBool = AtomicBool::new(false);
static COUNTER: AtomicU32 = AtomicU32::new(0);

fn callback(level: Level) {
    let last = LAST.load(Ordering::SeqCst);
    let current = level.eq(&Level::High);
    if current == last {
        COUNTER.fetch_add(1, Ordering::SeqCst);
    }
    LAST.store(current,Ordering::SeqCst);
}

fn main() -> rppal::gpio::Result<()> {
    let topo = Topology::new();

    // Check if Process Binding for CPUs is supported
    println!("CPU Binding (current process) supported: {}", topo.support().cpu().set_current_process());
    println!("CPU Binding (any process) supported: {}", topo.support().cpu().set_process());

    // Check if Thread Binding for CPUs is supported
    println!("CPU Binding (current thread) supported: {}", topo.support().cpu().set_current_thread());
    println!("CPU Binding (any thread) supported: {}", topo.support().cpu().set_thread());

    // Debug Print all the Support Flags
    println!("All Flags:\n{:?}", topo.support());

    set_self_priority(Process,-20).expect("TODO: panic message");
    let gpio = Gpio::new()?;
    let mut pin = gpio.get(15)?.into_input();
    pin.set_async_interrupt(Trigger::Both, &callback)?;
    let mut last_counter = 0;
    loop {
        let counter = COUNTER.load(Ordering::SeqCst);
        if counter - last_counter >= 10 {
            println!("wups! Missed {} edges!", counter);
            last_counter = counter;
            thread::sleep(Duration::from_millis(1000));
        }
    }
}

fn multithreading() {
    for i in 1..10 {
        println!("hi number {} from the spawned thread!", i);
        thread::sleep(Duration::from_millis(1));
    }
}

fn current_cpu() -> Result<usize, io::Error> {
    let ret = unsafe {
        libc::sched_getcpu()
    };

    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ret as usize)
    }
}
