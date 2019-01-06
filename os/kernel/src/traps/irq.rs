use pi::interrupt::Interrupt;

use traps::TrapFrame;
use process::State;
use pi::timer::tick_in;
use process::TICK;
use SCHEDULER;

pub fn handle_irq(interrupt: Interrupt, tf: &mut TrapFrame) {
    match interrupt {
        Interrupt::Timer1 => {
            tick_in(TICK);
            SCHEDULER.switch(State::Ready, tf).unwrap();
        }
        _ => unimplemented!("handle_irq()"),
    }
}
