use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Debug,
    sync::atomic::{AtomicUsize, Ordering},
};

type BoxedCallback<'a, T> = Box<dyn FnMut(&T) -> Response + 'a + Send>;
type SubscriptionId = usize;

static NEXT_SUBSCRIPTION_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Default)]
pub struct Delegate<'d, T> {
    pub(crate) subscriptions: RefCell<HashMap<SubscriptionId, BoxedCallback<'d, T>>>,
}

#[derive(Eq, Hash, PartialEq)]
pub struct Subscription {
    id: SubscriptionId,
}

pub enum Response {
    StaySubscribed,
    CancelSubscription,
}

impl<'d, T> Delegate<'d, T> {
    pub fn new() -> Self {
        Self {
            subscriptions: RefCell::new(HashMap::new()),
        }
    }

    pub fn subscribe<C: FnMut(&T) -> Response + 'd + Send>(&self, callback: C) -> Subscription {
        let id = NEXT_SUBSCRIPTION_ID.fetch_add(1, Ordering::SeqCst);
        let subscription = Subscription { id };
        self.subscriptions
            .borrow_mut()
            .insert(subscription.id, Box::new(callback));
        subscription
    }

    pub fn unsubscribe(&self, subscription: &Subscription) {
        self.subscriptions.borrow_mut().remove(&subscription.id);
    }

    pub fn broadcast(&self, value: &T) {
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
            match callback(value) {
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
    pub fn notify(&self) {
        self.broadcast(&());
    }
}

impl<T> Debug for Delegate<'_, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Delegate")
            .field(
                "subscriptions",
                &format_args!("{} active subscriptions", self.subscriptions.borrow().len()),
            )
            .finish()
    }
}
