use core::cell::Cell;

use crate::{
    executor::wake_task,
    future::{OurFuture, Poll},
};

pub struct Channel<T> {
    item: Cell<Option<T>>,
    task_id: Cell<Option<usize>>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            item: Cell::new(None),
            task_id: Cell::new(None),
        }
    }

    pub fn get_sender(&self) -> Sender<T> {
        Sender { channel: self }
    }

    pub fn get_receiver(&self) -> Receiver<T> {
        Receiver {
            channel: self,
            state: ReceiverState::Init,
        }
    }

    fn receive(&self) -> Option<T> {
        self.item.take()
    }

    fn send(&self, item: Option<T>) {
        self.item.replace(item);
        if let Some(task_id) = self.task_id.get() {
            wake_task(task_id);
        }
    }

    fn register(&self, task_id: usize) {
        self.task_id.replace(Some(task_id));
    }
}

pub struct Sender<'a, T> {
    channel: &'a Channel<T>,
}

impl<'a, T> Sender<'a, T> {
    pub fn send(&self, item: Option<T>) {
        self.channel.send(item)
    }
}

enum ReceiverState {
    Init,
    Wait,
}

pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
    state: ReceiverState,
}

impl<'a, T> Receiver<'a, T> {
    pub fn receive(&self) -> Option<T> {
        self.channel.receive()
    }
}

impl<T> OurFuture for Receiver<'_, T> {
    type Output = T;

    fn poll(&mut self, task_id: usize) -> Poll<Self::Output> {
        match self.state {
            ReceiverState::Init => {
                self.channel.register(task_id);
                self.state = ReceiverState::Wait;
                Poll::Pending
            }
            ReceiverState::Wait => match self.channel.receive() {
                Some(item) => Poll::Ready(item),
                None => Poll::Pending,
            },
        }
    }
}
