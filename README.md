# Squeak

[![build_badge]][build_link] [![crates.io_badge]][crates.io_link] [![docs_badge]][docs_link]

[build_badge]: https://img.shields.io/github/workflow/status/agersant/squeak/Continuous%20integration/master
[build_link]: https://github.com/agersant/squeak/actions/workflows/CI.yml?query=branch%3A+branch%3Amaster++
[crates.io_badge]: https://img.shields.io/badge/crates.io-squeak-green
[crates.io_link]: https://crates.io/crates/squeak
[docs_badge]: https://img.shields.io/badge/docs.rs-squeak-blue
[docs_link]: https://docs.rs/squeak/latest/squeak/

Squeak is a zero-dependency Rust library allowing execution of callbacks in response to values being broadcast or mutated.

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
