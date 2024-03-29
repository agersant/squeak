use alloc::vec::Vec;
use alloc::{borrow::Borrow, boxed::Box, collections::BTreeMap, fmt::Debug};

use core::cell::RefCell;
use core::sync::atomic::{AtomicU64, Ordering};

type BoxedCallback<'a, T> = Box<dyn FnMut(&T) -> Response + 'a + Send>;
type SubscriptionId = u64;

static NEXT_SUBSCRIPTION_ID: AtomicU64 = AtomicU64::new(0);

/// Maintains a list of callbacks that can be explicitely triggered
/// by calling [`Delegate::broadcast`].
#[derive(Default)]
pub struct Delegate<'d, T> {
    pub(crate) subscriptions: RefCell<BTreeMap<SubscriptionId, BoxedCallback<'d, T>>>,
}

/// Represents a subscription created via [`Delegate::subscribe`] or [`Observable::subscribe`](crate::Observable::subscribe).
///
/// It can be passed to [`Delegate::unsubscribe`] or [`Observable::unsubscribe`](crate::Observable::unsubscribe) to cancel the subscription.
#[derive(Eq, Hash, PartialEq)]
pub struct Subscription {
    id: SubscriptionId,
}

/// Returned by [`Delegate`] and [`Observable`](crate::Observable) subscription callbacks.
/// Depending on the value returned, the subscription will stay active or be cancelled.
pub enum Response {
    StaySubscribed,
    CancelSubscription,
}

impl<'d, T> Delegate<'d, T> {
    pub fn new() -> Self {
        Self {
            subscriptions: RefCell::new(BTreeMap::new()),
        }
    }

    /// Registers a new callback that will be called when this delegate broadcasts
    /// a new value.
    ///
    /// ```rust
    /// use squeak::{Delegate, Response};
    ///
    /// let on_damage_received = Delegate::new();
    /// on_damage_received.subscribe(|amount| {
    ///     println!("Received {amount} damage");
    ///     Response::StaySubscribed
    /// });
    /// on_damage_received.broadcast(5); // Prints "Received 5 damage"
    /// ```
    ///
    /// The output of the callback function determines whether it will be called
    /// again when [`broadcast`] is called in the future.
    ///
    pub fn subscribe<C: FnMut(&T) -> Response + 'd + Send>(&self, callback: C) -> Subscription {
        let id = NEXT_SUBSCRIPTION_ID.fetch_add(1, Ordering::SeqCst);
        let subscription = Subscription { id };
        self.subscriptions
            .borrow_mut()
            .insert(subscription.id, Box::new(callback));
        subscription
    }

    /// Removes a callback that was previously registered.
    ///
    /// ```rust
    /// use squeak::{Delegate, Response};
    ///
    /// let on_damage_received = Delegate::new();
    /// let subscription = on_damage_received.subscribe(|amount| {
    ///     println!("Received {amount} damage");
    ///     Response::StaySubscribed
    /// });
    /// on_damage_received.broadcast(5); // Prints "Received 5 damage"
    /// on_damage_received.unsubscribe(subscription);
    /// on_damage_received.broadcast(10); // Does not print anything
    /// ```
    /// - Attempting to unsubscribe using a [`Subscription`] that was created by a different [`Delegate`] has no effect.
    /// - Attempting to unsubscribe a [`Subscription`] multiple times has no effect.
    /// - Attempting to unsubscribe from within callback function has no effect.
    pub fn unsubscribe(&self, subscription: Subscription) {
        self.subscriptions.borrow_mut().remove(&subscription.id);
    }

    /// Executes all registered callbacks, providing `value` as their argument.
    ///
    /// ```rust
    /// use squeak::{Delegate, Response};
    ///
    /// let on_renamed = Delegate::new();
    /// on_renamed.subscribe(|new_name: &String| {
    ///     println!("New name is {new_name}");
    ///     Response::StaySubscribed
    /// });
    /// on_renamed.broadcast(String::from("Lisa"));
    /// on_renamed.broadcast(&String::from("Trevor"));
    /// on_renamed.broadcast(&mut String::from("Jill"));
    /// ```
    pub fn broadcast<U: Borrow<T>>(&self, value: U) {
        let subscriptions_to_notify = self
            .subscriptions
            .borrow()
            .keys()
            .copied()
            .collect::<Vec<_>>();
        for subscription in subscriptions_to_notify {
            let (_, mut callback) = self
                .subscriptions
                .borrow_mut()
                .remove_entry(&subscription)
                .unwrap();
            match callback(value.borrow()) {
                Response::CancelSubscription => (),
                Response::StaySubscribed => {
                    self.subscriptions
                        .borrow_mut()
                        .insert(subscription, callback);
                }
            };
        }
    }
}

impl Delegate<'_, ()> {
    /// This convenience function broadcasts the unit type on delegates with no payload.
    ///
    /// ```rust
    /// use squeak::{Delegate, Response};
    ///
    /// let on_respawn = Delegate::new();
    /// on_respawn.subscribe(|_| {
    ///    println!("Respawned");
    ///    Response::StaySubscribed
    /// });
    /// on_respawn.notify();
    /// ```
    pub fn notify(&self) {
        self.broadcast(());
    }
}

impl<T> Debug for Delegate<'_, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        f.debug_struct("Delegate")
            .field(
                "subscriptions",
                &format_args!("{} active subscriptions", self.subscriptions.borrow().len()),
            )
            .finish()
    }
}
