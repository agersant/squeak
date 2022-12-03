use squeak::{Observable, Response};

#[test]
fn observable_broadcasts_new_values() {
    let mut seen_value = 0;
    {
        let mut o = Observable::new(0);
        o.subscribe(|new_value| {
            seen_value = *new_value;
            Response::StaySubscribed
        });
        o.mutate(|value| *value = 42);
    }
    assert_eq!(seen_value, 42);
}

#[test]
fn observable_no_longer_notifies_after_unsubscribe() {
    let mut call_count = 0;
    {
        let mut o = Observable::new(0);
        let s = o.subscribe(|_| {
            call_count += 1;
            Response::StaySubscribed
        });
        o.mutate(|value| *value = 42);
        o.unsubscribe(&s);
        o.mutate(|value| *value = 43);
    }
    assert_eq!(call_count, 1);
}
