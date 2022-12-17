use parking_lot::{Mutex, ReentrantMutex};
use std::{cell::RefCell, ops::Deref, sync::Arc};

use squeak::{Delegate, Response};

#[test]
fn delegate_executes_callbacks() {
    let mut call_count = 0;
    {
        let d = Delegate::new();
        d.subscribe(|_| {
            call_count += 1;
            Response::StaySubscribed
        });
        d.notify();
        d.notify();
        d.notify();
    }
    assert_eq!(call_count, 3);
}

#[test]
fn can_subscribe_during_callback() {
    let d = Arc::new(ReentrantMutex::new(Delegate::new()));
    let outer_count = Arc::new(Mutex::new(RefCell::new(0)));
    let inner_count = Arc::new(Mutex::new(RefCell::new(0)));
    {
        let d_clone = d.clone();
        let outer_count_clone = outer_count.clone();
        let inner_count_clone = inner_count.clone();
        d.lock().subscribe(move |_| {
            *outer_count_clone.lock().borrow_mut() += 1;
            let inner_count_clone_clone = inner_count_clone.clone();
            d_clone.lock().subscribe(move |_| {
                *inner_count_clone_clone.lock().borrow_mut() += 1;
                Response::CancelSubscription
            });
            Response::CancelSubscription
        });
        d.lock().notify();
        d.lock().notify();
    }
    assert_eq!(*outer_count.lock().borrow(), 1);
    assert_eq!(*inner_count.lock().borrow(), 1);
}

#[test]
fn delegate_does_not_execute_unsubscribed_callbacks() {
    let mut call_count = 0;
    {
        let d = Delegate::new();
        let subscription = d.subscribe(|_| {
            call_count += 1;
            Response::StaySubscribed
        });
        d.unsubscribe(subscription);
        d.notify();
    }
    assert_eq!(call_count, 0);
}

#[test]
fn cannot_unsubscribe_using_subscription_from_a_different_delegate() {
    let mut call_count = 0;
    {
        let d1 = Delegate::<()>::new();
        let d2 = Delegate::<()>::new();
        let _s1 = d1.subscribe(|_| {
            call_count += 1;
            Response::StaySubscribed
        });
        let s2 = d2.subscribe(|_| Response::StaySubscribed);
        d1.unsubscribe(s2);
        d1.notify();
    }
    assert_eq!(call_count, 1);
}

#[test]
fn unsubscribing_within_callback_is_noop() {
    let d = Arc::new(ReentrantMutex::new(Delegate::new()));
    let call_count = Arc::new(Mutex::new(RefCell::new(0)));
    let subscription = Arc::new(Mutex::new(RefCell::new(None)));

    let d_clone = d.clone();
    let call_count_clone = call_count.clone();
    let subscription_clone = subscription.clone();

    subscription
        .lock()
        .replace(Some(d.lock().subscribe(move |_| {
            let old_count = *call_count_clone.lock().borrow();
            *call_count_clone.lock().borrow_mut() = old_count + 1;
            if let Some(subscription) = subscription_clone.lock().deref().borrow_mut().take() {
                d_clone.lock().unsubscribe(subscription);
            }
            Response::StaySubscribed
        })));

    d.lock().notify();
    d.lock().notify();
    assert_eq!(*call_count.lock().borrow(), 2);
}

#[test]
fn can_unsubscribe_using_response_value() {
    let mut call_count = 0;
    {
        let d = Delegate::new();
        d.subscribe(|_| {
            call_count += 1;
            Response::CancelSubscription
        });
        d.notify();
        d.notify();
    }
    assert_eq!(call_count, 1);
}
