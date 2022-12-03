Squeak is a zero-dependency Rust library defining types which help executing callbacks in response to values being broadcast or mutated.

# Examples

```rust
use squeak::{Delegate, Response};

let on_damage_received = Delegate::new();
on_damage_received.subscribe(|amount| {
    println!("Received {amount} damage");
    Response::StaySubscribed
});

on_damage_received.broadcast(16); // Prints "Received 16 damage"
on_damage_received.broadcast(14); // Prints "Received 14 damage"
on_damage_received.broadcast(28); // Prints "Received 28 damage"
```

```rust
use squeak::{Observable, Response};

let mut health = Observable::new(100);
health.subscribe(|updated_health| {
    println!("Health is now {updated_health}");
    Response::StaySubscribed
});

health.mutate(|h| *h -= 10); // Prints "Health is now 90"
health.mutate(|h| *h -= 5);  // Prints "Health is now 85"
health.mutate(|h| *h += 25); // Prints "Health is now 110"
```
