use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::pin;
use tokio::task::{JoinError, JoinHandle};

pub struct TryJoinHandle<T> {
    join: JoinHandle<T>,
}

impl<T> TryJoinHandle<T> {
    pub fn check(&mut self) -> TryJoin<T> {
        TryJoin {
            join: &mut self.join,
        }
    }
}

impl<T> Deref for TryJoinHandle<T> {
    type Target = JoinHandle<T>;

    fn deref(&self) -> &Self::Target {
        &self.join
    }
}

impl<'a, T> DerefMut for TryJoinHandle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.join
    }
}

impl<'a, T> From<JoinHandle<T>> for TryJoinHandle<T> {
    fn from(join: JoinHandle<T>) -> Self {
        TryJoinHandle { join }
    }
}

pub struct TryJoin<'a, T> {
    join: &'a mut JoinHandle<T>,
}

impl<'a, T> Future for TryJoin<'a, T> {
    type Output = Poll<Result<T, JoinError>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let join = Pin::new(self.join);

        Poll::Ready(Future::poll(join, cx))
    }
}
