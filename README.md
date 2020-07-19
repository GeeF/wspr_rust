# wspr_rust
A WSPR (Weak Signal Propagation Reporter) encoder written in Rust.

It does encode the standard WSPR messages (or Type 1) for now. It is fully tested against the encoding of the reference implementation using automatically generated test messages.

## Example:
```rust
/// usage example
use wspr::WSPRMessage;

let wspr_msg = WSPRMessage::new("DB2LA", "JO51", 30);
let encoded = wspr_msg.encode().unwrap();
```

The encoded data is a ``u8`` array of size 162 that represents the channel encoded symbols where ``0`` is the first frequency and ``3`` is the fourth frequency of the 4-fsk encoding. You can use this data to control an oscillator directly or synthesize an audio baseband signal as an input for a TX modulating SSB.

 A few notes for now:
* Type 2/3 messages which can encode extended callsigns and a more precise grid locator are a bit sparsely documented. My personal need for them is not high enough to justify digging through the Fortran code of the reference implementation :)

TODO:
* Add impl.md and document a few implementation details