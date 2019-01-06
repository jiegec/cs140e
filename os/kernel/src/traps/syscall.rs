use traps::TrapFrame;
use pi::timer::current_time;
use SCHEDULER;
use process::State;
use process::Process;

/// Sleep for `ms` milliseconds.
///
/// This system call takes one parameter: the number of milliseconds to sleep.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the approximate true elapsed time from when `sleep` was called to
/// when `sleep` returned.
pub fn sleep(ms: u32, tf: &mut TrapFrame) {
    let begin = current_time();
    let time = begin + ms as u64 * 1000;
    let polling_fn = Box::new(move |process: &mut Process| {
        let current = current_time();
        if current > time {
            process.trap_frame.x1to29[6] = 0; // x7 = 0; succeed
            process.trap_frame.x0 = (current - begin) / 1000; // x0 = elapsed time in ms
            true
        } else {
            false
        }
    });
    SCHEDULER.switch(State::Waiting(polling_fn), tf).unwrap();
}

pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    match num {
        1 => {
            sleep(tf.x0 as u32, tf);
        }
        _ => {
            tf.x1to29[6] = 1; // x7 = 1, do not exist
        }
    }
}
