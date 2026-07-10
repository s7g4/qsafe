# Q-Safe

{{#include ../../../README.md:intro}}

## Demo

![Q-Safe demo: register two users, reject a wrong password, log in, exchange a message over the authenticated WebSocket, and run the full test suite](assets/demo.gif)

Every line of output above is a real capture (`docker`, `curl`, `cargo run`,
`cargo test`) against a locally running instance in Mock HSM mode - nothing
staged. Typing is sped up; command output is not edited except truncating
JWTs for readability.

{{#include ../../../README.md:whats-tested}}
