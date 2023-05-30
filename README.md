# Lightweight generic IO

Less error-prone, `no_std` IO traits generic over their error types.

This is a simplified version of [`genio`](https://docs.rs/genio) containing only buffered
traits.

## Advantages over `std`

* `no_std` - doesn't require an operating system
* Error being associated type is more flexible
* Less error-prone - no `read` and `write` methods which are often mistaken for `read_all` or
  `write_all`

## Advantages over `genio`

* Simpler
* Less `unsafe` to deal with uninitialized bytes (currently none, may change in the future)
* Most uses of IO need some buffering anyway
* Less error-prone - no `read` and `write` methods which are often mistaken for `read_all` or
  `write_all`
* No `FlushError` makes error handling simpler

## Target audience

Mainly serialization libraries and their consumers.
Can be also usful for simple protocols that don't need precise control of `read` and `write`
calls.

Probably should *not* be used in lower layers.

## Usage overview

The [`BufRead`] trait is very similar to the one from `std`. The biggest differences are error
type and lack of error-prone `read` method. Since it is implemented on `std::io::BufReader` and
primitive `std` types you can use it exactly the same as [`std::io::BufRead`] in most cases.
There's an added benefit that you can statically prove reading from `&[u8]` will not fail (but
it can return `UnexpectedEnd`).

Similarly, [`BufWrite`] is just [`std::io::Write`] with error being associated and missing
`write` method. It still requires that writing is either buffered or fast because that's what
most encoders need.

## Features

* `std` - integration with the standard library: implementations and adapters
* `alloc` - additional features requiring allocation

## MSRV

The crate intends to have conservative MSRV and only bump it when it provides significant
benefit and at most to the version available in latest Debian stable. Currently tested MSRV is
1.41.1 (Debian oldstable) but due to its simplicity it's possible it works on even lower
versions.

Some features may be only available in newer Rust versions. Thus it is recommended to use
recent Rust if possible.
