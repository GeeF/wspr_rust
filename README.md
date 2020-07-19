# wspr_rust
A WSPR (Weak Signal Propagation Reporter) encoder written in Rust

It does encode the standard WSPR messages (or Type 1) for now. It is fully tested against the encoding of the reference implementation using automatically generated test messages.

 A few notes for now:
* Type 2/3 messages which can encode extended callsigns and a more precise grid locator are a bit sparsely documented. My personal need for them is not high enough to justify digging through the Fortran code of the reference implementation :)

TODO:
* Add impl.md and document a few implementation details