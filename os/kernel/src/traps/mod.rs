mod irq;
mod trap_frame;
mod syndrome;
mod syscall;

use pi::interrupt::{Controller, Interrupt};

pub use self::trap_frame::TrapFrame;

use console::kprintln;
use self::syndrome::Syndrome;
use self::irq::handle_irq;
use self::syscall::handle_syscall;
use user::shell as shell;

#[cfg(feature = "qemu")]
use pi::timer::Timer;

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Source {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Info {
    source: Source,
    kind: Kind,
}

/// This function is called when an exception occurs. The `info` parameter
/// specifies the source and kind of exception that has occurred. The `esr` is
/// the value of the exception syndrome register. Finally, `tf` is a pointer to
/// the trap frame for the exception.
#[no_mangle]
pub extern "C" fn handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) {
    let syndrome = Syndrome::from(esr);
    if info.kind == Kind::Synchronous {
        // syndrome is only valid with sync
        match syndrome {
            Syndrome::Brk(_) => {
                kprintln!("Got {:?} from {:?}", syndrome, info.source);
                shell(" [brk]$ ");

                tf.elr += 4; // Skip the current brk instruction
                return;
            }
            Syndrome::Svc(syscall) => {
                handle_syscall(syscall, tf);
                return;
            }
            _ => {}
        }
    } else if info.kind == Kind::Irq {
        let controller = Controller::new();
        use self::Interrupt::*;
        for interrupt in [Timer1, Timer3, Usb, Gpio0, Gpio1, Gpio2, Gpio3, Uart].iter() {
            if controller.is_pending(*interrupt) {
                handle_irq(*interrupt, tf);
                return;
            }
        }

        #[cfg(feature = "qemu")]
        {
            if Timer::new().is_pending() {
                handle_irq(Timer1, tf);
                return;
            }
        }
    }
    kprintln!(
        "Unhandled exception with info: {:?}, syndrome: {:?}, tf: {:?}",
        info,
        syndrome,
        tf
    );
    loop {
        #[cfg(not(test))]
        unsafe { asm!("wfe") }
    }
}
