use cortex_m::asm;
use heapless::mpmc::Q4;
use rtt_target::rprintln;

use crate::future::OurFuture;

static TASK_QUEUE: Q4<usize> = Q4::new();

pub fn wake_task(task_id: usize) {
    rprintln!("Waking task {}", task_id);
    if TASK_QUEUE.enqueue(task_id).is_err() {
        panic!("Task queue full: cannnot add task {}", task_id);
    }
}

pub fn run_tasks(tasks: &mut [&mut dyn OurFuture<Output = ()>]) -> ! {
    for task_id in 0..tasks.len() {
        TASK_QUEUE.enqueue(task_id).unwrap();
    }

    loop {
        while let Some(task_id) = TASK_QUEUE.dequeue() {
            if task_id > tasks.len() {
                rprintln!("Invalid task {}!", { task_id });
                continue;
            };
            rprintln!("Running task {}", task_id);
            tasks[task_id].poll(task_id);
        }
        rprintln!("No tasks ready, going to sleep...");
        asm::wfi();
    }
}
