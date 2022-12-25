# Drips
[![crates.io](https://img.shields.io/crates/v/drips.svg)](https://crates.io/crates/drips)

## Usage

```bash
drips listen PORT           -- on the listening computer
drips send ADDRESS FILE     -- on the sending computer
```
A cli for sending and receiving files over TCP.


## Install

### Install with `cargo install`

```bash
$ cargo install drips
```

### Build from source

```bash
$ git clone https://github.com/zschaffer/drips
$ cd drips
$ cargo build --release
```
