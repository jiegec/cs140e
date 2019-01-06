#[cfg(not(test))]
pub fn sleep(time: u64) -> (u64, u64) {
    let error: u64;
    let time_elapsed: u64;
    unsafe {
        asm!("mov x0, $2
              svc 1
              mov $0, x0
              mov $1, x7"
              : "=r"(time_elapsed), "=r"(error)
              : "r"(time)
              : "x0", "x7")
    };
    (error, time_elapsed)
}

#[cfg(test)]
pub fn sleep(_: u64) -> (u64, u64) {
    (0, 0)
}