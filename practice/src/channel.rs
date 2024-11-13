use core::cell::Cell;

pub struct Channel<T> {
    item: Cell<Option<T>>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            item: Cell::new(None),
        }
    }

    pub fn get_sender(&self) -> Sender<T> {
        Sender { channel: self }
    }

    pub fn get_receiver(&self) -> Receiver<T> {
        Receiver { channel: self }
    }

    fn receive(&self) -> Option<T> {
        self.item.take()
    }

    fn send(&self, item: Option<T>) {
        self.item.replace(item);
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

pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
}

impl<'a, T> Receiver<'a, T> {
    pub fn receive(&self) -> Option<T> {
        self.channel.receive()
    }
}
