[mio] is a Rust async I/O library I use in the [Rust version] of the [gnirehtet]
relay server.

[mio]: https://github.com/carllerche/mio
[Rust version]: https://github.com/Genymobile/gnirehtet/tree/rust/rustrelay
[gnirehtet]: https://github.com/Genymobile/gnirehtet

However, it does not work on Windows, because the behavior I observe is
unexpected to me. Thus, I wrote this minimal sample to get feedbacks about the
expected behavior.


## Principle

This sample starts listening on a (blocking) TCP socket on 127.0.0.1:1234 in a
separate thread.

Then it connects to it using a (non-blocking / mio) TCP socket, and poll to get
the incoming events.

The server then sends some data to the client. On events, the client prints data
on _stdout_.

Here is the expected behavior:

```
                        client      server
                          |           |
                          |           *  server starts listening on 127.0.0.1
         client connects  >-----      *
            client polls  *     \     *
                          *      ----->  client accepted
                          *           |
                          *      -----<  server writes "Hello"
                          *     / ----<  server writes ", "
                readable  <----- /    |
                          <------     .  server sleeps for 1 second
                          |           .
  client reads "Hello, "  R           .
                          *           .
                          *      -----<  server writes "world!"
                          *     / ---<_  and closes
                readable  <----- /
                          <------
                          |
   client reads "world!"  R
                          |
                readable  *
        client reads EOF  R
              and closes  _
```

## Linux

On Linux (Debian 9), it behaves as expected.

Compile with _Cargo_:

    cargo build

Then run the generated binary:

    target/debug/mio-sample

Here is the output:

```
SERVER (sender): listening on 127.0.0.1:1234
SERVER (sender): writing 'Hello'
SERVER (sender): writing ', '
CLIENT (receiver): event=Event { kind: Ready {Readable}, token: Token(0) }
CLIENT (receiver): read 7 bytes: [Hello, ]
SERVER (sender): writing 'world!'
SERVER (sender): closing
CLIENT (receiver): event=Event { kind: Ready {Readable}, token: Token(0) }
CLIENT (receiver): read 6 bytes: [world!]
CLIENT (receiver): event=Event { kind: Ready {Readable}, token: Token(0) }
CLIENT (receiver): eof
```


## Windows

On Windows, however, sometimes `poll()` generates a readable event, but calling
`read()` on the stream fails with `WouldBlock`, which seems inconsistent to me.

Compile with _Cargo_:

    cargo build

Then run the generated binary:

    target\debug\mio-sample.exe

Here is the output:

```
SERVER (sender): listening on 127.0.0.1:1234
SERVER (sender): writing 'Hello'
SERVER (sender): writing ', '
CLIENT (receiver): event=Event { kind: Ready {Readable}, token: Token(0) }
CLIENT (receiver): read 7 bytes: [Hello, ]
CLIENT (receiver): event=Event { kind: Ready {Readable}, token: Token(0) }
CLIENT (receiver): error [WouldBlock]: A non-blocking socket operation could not
be completed immediately. (os error 10035)
```

### Workaround

As a workaround, we can ignore any `WouldBlock` error on Windows.

    cargo build --features workaround
    target\debug\mio-sample.exe

Here is the output:

```
SERVER (sender): listening on 127.0.0.1:1234
SERVER (sender): writing 'Hello'
SERVER (sender): writing ', '
CLIENT (receiver): event=Event { kind: Ready {Readable}, token: Token(0) }
CLIENT (receiver): read 7 bytes: [Hello, ]
CLIENT (receiver): event=Event { kind: Ready {Readable}, token: Token(0) }
CLIENT (receiver): spurious event, ignoring
SERVER (sender): writing 'world!'
SERVER (sender): closing
CLIENT (receiver): event=Event { kind: Ready {Readable}, token: Token(0) }
CLIENT (receiver): read 6 bytes: [world!]
CLIENT (receiver): event=Event { kind: Ready {Readable}, token: Token(0) }
CLIENT (receiver): eof
```

However, this is very intrusive, because it has to be added on every place we
read or write data, in code that assumed that reading or writing may not fail
(we just called `poll()`!).


### Unexpected behavior

Here is the description of the [error 10035] reported by Windows:

> Resource temporarily unavailable.
> 
> This error is returned from operations on nonblocking sockets that cannot be
> completed immediately, for example recv when no data is queued to be read
> from the socket. It is a nonfatal error, and the operation should be retried
> later. **It is normal for WSAEWOULDBLOCK to be reported as the result from
> calling connect on a nonblocking SOCK_STREAM socket, since some time must
> elapse for the connection to be established.**

[error 10035]: https://msdn.microsoft.com/en-us/library/windows/desktop/ms740668(v=vs.85).aspx

However, in this sample, `WouldBlock` results from a spurious readable event,
not related to the connect call (the client has already read the first readable
event).
