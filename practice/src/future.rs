pub trait OurFuture {
    type Output;

    fn poll(&mut self, task_id: usize) -> Poll<Self::Output>; // TODO: Why should this be non-`pub`?
}

pub enum Poll<T> {
    Pending,
    Ready(T),
}
