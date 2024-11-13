use embedded_hal::digital::{OutputPin, StatefulOutputPin};
use fugit::ExtU64;
use microbit::{
    gpio::NUM_COLS,
    hal::gpio::{Output, Pin, PushPull},
};
use rtt_target::rprintln;

use crate::{button::ButtonDirection, channel::Receiver, time::Timer};

enum LedState {
    Toggle,
    Wait(Timer),
}

pub struct LedTask<'a> {
    cols: [Pin<Output<PushPull>>; NUM_COLS],
    active_col: usize,
    state: LedState,
    receiver: Receiver<'a, ButtonDirection>,
}

impl<'a> LedTask<'a> {
    pub fn new(
        col: [Pin<Output<PushPull>>; NUM_COLS],
        receiver: Receiver<'a, ButtonDirection>,
    ) -> Self {
        Self {
            cols: col,
            active_col: 0,
            state: LedState::Toggle,
            receiver,
        }
    }

    pub fn poll(&mut self) {
        match self.state {
            LedState::Toggle => {
                self.toggle();
                let timer = Timer::new(500.millis());
                self.state = LedState::Wait(timer);
            }
            LedState::Wait(ref timer) => {
                if timer.is_ready() {
                    self.state = LedState::Toggle;
                }
                if let Some(direction) = self.receiver.receive() {
                    self.cols[self.active_col].set_high().ok();
                    self.shift(direction);
                    self.state = LedState::Toggle;
                }
            }
        }
    }

    fn toggle(&mut self) -> Option<()> {
        rprintln!("Blinking LED {}", self.active_col);
        #[cfg(feature = "trigger-overflow")]
        {
            use crate::time::Ticker;
            let time = Ticker::now();
            rprintln!(
                "Time: 0x{:x} ticks, {} ms",
                time.ticks(),
                time.duration_since_epoch().to_millis(),
            )
        }
        self.cols[self.active_col].toggle().ok()
    }

    fn shift(&mut self, direction: ButtonDirection) {
        self.active_col = match direction {
            ButtonDirection::Left => match self.active_col {
                0 => NUM_COLS - 1,
                _ => self.active_col - 1,
            },
            ButtonDirection::Right => (self.active_col + 1) % NUM_COLS,
        };
        // switch off new LED: moving to Toggle will then switch it on
        self.cols[self.active_col].set_high().ok();
    }
}
