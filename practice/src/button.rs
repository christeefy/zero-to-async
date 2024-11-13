use embedded_hal::digital::InputPin;
use fugit::ExtU64;
use microbit::hal::gpio::{Floating, Input, Pin};
use rtt_target::rprintln;

use crate::{channel::Sender, time::Timer};

#[derive(Debug, Copy, Clone)]
pub enum ButtonDirection {
    Left,
    Right,
}

enum ButtonState {
    WaitForPress,
    Debounce(Timer),
}

pub struct ButtonTask<'a> {
    pin: Pin<Input<Floating>>,
    state: ButtonState,
    direction: ButtonDirection,
    sender: Sender<'a, ButtonDirection>,
}

impl<'a> ButtonTask<'a> {
    pub fn new(
        pin: Pin<Input<Floating>>,
        direction: ButtonDirection,
        sender: Sender<'a, ButtonDirection>,
    ) -> Self {
        Self {
            pin,
            state: ButtonState::WaitForPress,
            direction,
            sender,
        }
    }

    pub fn poll(&mut self) {
        match self.state {
            ButtonState::WaitForPress => {
                if self.pin.is_low().unwrap() {
                    rprintln!("{:#?} button pressed", self.direction);
                    self.sender.send(Some(self.direction));
                    self.state = ButtonState::Debounce(Timer::new(100.millis()));
                }
            }
            ButtonState::Debounce(ref timer) => {
                if timer.is_ready() && self.pin.is_high().unwrap() {
                    self.state = ButtonState::WaitForPress;
                }
            }
        }
    }
}
