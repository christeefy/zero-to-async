use embedded_hal::digital::PinState;
use fugit::ExtU64;
use microbit::hal::{
    gpio::{Floating, Input, Pin},
    gpiote::Gpiote,
};
use rtt_target::rprintln;

use crate::{
    channel::Sender,
    future::{OurFuture, Poll},
    gpiote::InputChannel,
    time::Timer,
};

#[derive(Debug, Copy, Clone)]
pub enum ButtonDirection {
    Left,
    Right,
}

enum ButtonState {
    WaitForPress,
    Debounce(Timer),
    WaitForRelease,
}

pub struct ButtonTask<'a> {
    input: InputChannel,
    state: ButtonState,
    direction: ButtonDirection,
    sender: Sender<'a, ButtonDirection>,
}

impl<'a> ButtonTask<'a> {
    pub fn new(
        pin: Pin<Input<Floating>>,
        direction: ButtonDirection,
        sender: Sender<'a, ButtonDirection>,
        gpiote: &Gpiote,
    ) -> Self {
        Self {
            input: InputChannel::new(pin, &gpiote),
            state: ButtonState::WaitForPress,
            direction,
            sender,
        }
    }
}

impl OurFuture for ButtonTask<'_> {
    type Output = ();

    fn poll(&mut self, task_id: usize) -> Poll<Self::Output> {
        loop {
            match self.state {
                ButtonState::WaitForPress => {
                    self.input.set_ready_state(PinState::Low);
                    if let Poll::Ready(_) = self.input.poll(task_id) {
                        rprintln!("{:#?} button pressed", self.direction);
                        self.sender.send(Some(self.direction));
                        self.state = ButtonState::Debounce(Timer::new(100.millis()));
                        continue;
                    }
                }
                ButtonState::Debounce(ref timer) => {
                    if timer.is_ready() {
                        self.state = ButtonState::WaitForRelease;
                        continue;
                    }
                }
                ButtonState::WaitForRelease => {
                    self.input.set_ready_state(PinState::High);
                    if let Poll::Ready(_) = self.input.poll(task_id) {
                        self.state = ButtonState::WaitForPress;
                        continue;
                    }
                }
            }
            break;
        }
        Poll::Pending
    }
}
