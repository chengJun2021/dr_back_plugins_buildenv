use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::sync::mpsc::Receiver;

pub struct TryReceiver<T> {
    rx: Receiver<T>
}

impl<T> TryReceiver<T> {
    pub fn try_recv(&mut self) -> TryItem<T> {
        TryItem { rx: &mut self.rx }
    }
}

impl<T> Deref for TryReceiver<T> {
    type Target = Receiver<T>;

    fn deref(&self) -> &Self::Target {
        &self.rx
    }
}

impl<T> DerefMut for TryReceiver<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rx
    }
}

impl<T> From<Receiver<T>> for TryReceiver<T> {
    fn from(rx: Receiver<T>) -> Self {
        TryReceiver { rx }
    }
}

pub struct TryItem<'a, T> {
    rx: &'a mut Receiver<T>
}

impl<'a, T> Future for TryItem<'a, T> {
    type Output = Option<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.rx.poll_recv(cx) {
            Poll::Pending => Poll::Ready(None),
            Poll::Ready(t) => Poll::Ready(t),
        }
    }
}