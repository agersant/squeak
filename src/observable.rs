use std::{fmt::Debug, ops::Deref};

use crate::{Delegate, Response, Subscription};

#[derive(Debug)]
pub struct Observable<'o, T> {
    value: T,
    delegate: Delegate<'o, T>,
}

impl<'o, T> Observable<'o, T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            delegate: Delegate {
                subscriptions: Default::default(),
            },
        }
    }

    pub fn subscribe<C: FnMut(&T) -> Response + 'o + Send>(&self, callback: C) -> Subscription {
        self.delegate.subscribe(callback)
    }

    pub fn unsubscribe(&self, subscription: &Subscription) {
        self.delegate.unsubscribe(subscription);
    }

    pub fn delegate(&self) -> &Delegate<'o, T> {
        &self.delegate
    }

    pub fn mutate<M>(&mut self, mutation: M)
    where
        M: FnOnce(&mut T),
    {
        mutation(&mut self.value);
        self.delegate.broadcast(&self.value);
    }
}

impl<T> Default for Observable<'_, T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> Deref for Observable<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
