# serde-xdr

## Overview 
This library is taken and modified from `rust-xdr` by Ben Brittain. The original source can be found [here](https://github.com/benbrittain/rust-xdr/tree/master/src/serde_xdr).

The primary reason for "forking" was to be able to include it another project, `rusty-vxi11` as well as make modifications to it as necessary.
For instance, the serializer and deserializer were mostly rewritten to support the latest version of serde. There's also plans to add some tests.

## Building

`cargo build`

## Testing
`cargo test`
