[![GHA Build Status](https://github.com/rednithin/rossa/workflows/Rust/badge.svg?branch=master&event=push)](https://github.com/rednithin/rossa/actions?query=workflow%3ARust)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![crates.io](https://img.shields.io/crates/v/rossa.svg)](https://crates.io/crates/rossa)
[![Released API docs](https://docs.rs/rossa/badge.svg)](https://docs.rs/rossa)

# rossa

Attempt to create file server like SimpleHTTPServer in Rust. Also inspired by gossa.

## Installation

```
cargo install rossa
```

## Usage

```
rossa
```

### To use with custom host and port

```
rossa -a <host>:<port>
```

> Example

```
rossa -a 127.0.0.1:3333
```

## In Action

![Gif](https://i.makeagif.com/media/5-26-2020/bLQ17-.gif)

## Inspired By

- [https://github.com/pldubouilh/gossa](Gossa)
- [https://docs.python.org/2/library/simplehttpserver.html](SimpleHTTPServer)

## Similar Software

- [https://github.com/brson/basic-http-server](basic-http-server)

## Technologies Used

| Technology   | Purpose                                          |
| ------------ | ------------------------------------------------ |
| `warp`       | Web Framework                                    |
| `tokio`      | Async `fs`                                       |
| `tera`       | Templating                                       |
| `clap`       | CLI Interface                                    |
| `rand`       | To generate random files prefix                  |
| `rust-embed` | To embed assets and templates into single binary |
