use std::{fmt::Debug, ops::Deref};

use crate::{Delegate, Response, Subscription};

/// Observables are a wrapper type which owns a value, and executes subscription callbacks
/// every time a call is made to change the value.
///
/// ``` rust
/// use squeak::{Observable, Response};
///
/// let mut health = Observable::new(100);
/// health.subscribe(|updated_health| {
///     println!("Health is now {updated_health}");
///     Response::StaySubscribed
/// });
///
/// health.mutate(|h| *h -= 10); // Prints "Health is now 90"
/// health.mutate(|h| *h -= 5);  // Prints "Health is now 85"
/// health.mutate(|h| *h += 25); // Prints "Health is now 110"
/// ```
///
/// Observables implement [`std::ops::Deref`], which means the inner value can be accessed
/// via `*my_observable`.
#[derive(Debug)]
pub struct Observable<'o, T> {
    value: T,
    delegate: Delegate<'o, T>,
}

impl<'o, T> Observable<'o, T> {
    /// Creates a new observable with an initial value
    /// ```rust
    /// use squeak::Observable;
    /// let name = Observable::new(String::from("DefaultName"));
    /// ```
    pub fn new(value: T) -> Self {
        Self {
            value,
            delegate: Delegate {
                subscriptions: Default::default(),
            },
        }
    }

    /// Registers a new callback that will be called when the value contained in this observable is mutated.
    /// ```rust
    /// use squeak::{Observable, Response};
    ///
    /// let mut health = Observable::new(100);
    /// health.subscribe(|updated_health| {
    ///     println!("Health is now {updated_health}");
    ///     Response::StaySubscribed
    /// });
    /// ```
    pub fn subscribe<C: FnMut(&T) -> Response + 'o + Send>(&self, callback: C) -> Subscription {
        self.delegate.subscribe(callback)
    }

    /// Removes a callback that was previously registered.
    /// ```rust
    /// use squeak::{Observable, Response};
    ///
    /// let mut health = Observable::new(100);
    /// let subscription = health.subscribe(|updated_health| {
    ///     println!("Health is now {updated_health}");
    ///     Response::StaySubscribed
    /// });
    /// health.unsubscribe(subscription);
    /// ```
    pub fn unsubscribe(&self, subscription: Subscription) {
        self.delegate.unsubscribe(subscription);
    }

    /// Returns a reference to a delegate that will execute subscription functions
    /// when the observable is mutated. This is useful when writing a struct with
    /// a observable member, where users of the struct can subscribe to updates
    /// but not directly access the observable.
    pub fn delegate(&self) -> &Delegate<'o, T> {
        &self.delegate
    }

    /// Execute a function which may mutate the value contained in this observable.
    /// Subscription callbacks will be executed regardless of what happens inside
    /// the `mutation` function.
    /// ```rust
    /// use squeak::Observable;
    ///
    /// let name = Observable::new(String::from("DefaultName"));
    /// name.mutate(|n| n.append("X"));
    /// name.mutate(|n| n.append("Y"));
    /// name.mutate(|n| n.append("Z"));
    /// assert_eq!(*name.as_str(), "DefaultNameXYZ");
    /// ```
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
