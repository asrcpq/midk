midi sucks, although I am not planning to propose a comprehensive protocol to replace it,
I just want to work on a dead simple format: usm, to simplify my softwares.
Even there is a comprehensive format in the future,
usm is so simple that working on compatibility will be very easy.

Basically you should not handle protocol negotiation inside usm.

format:

```
document = message1 message2 ...
// all little endian
message = dt(microsecond):u32 message_type(mt):u32 length(len):u32 content:Vec<u8>
```

content spec example(midk, specialized for keyboard):

```
mt: 1, note up, content: key: u8, velocity: f32(0.0-1.0)
mt: 2, note off, content: key: u8
mt: 3, sustain, content: on_off: bool
mt: 4, beat, content: is_measure: bool
mt: 5, marker, content: id: String
```
