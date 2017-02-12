# rust-immutable-seq [![](http://meritbadge.herokuapp.com/immutable-seq)](https://crates.io/crates/immutable-seq)[![](https://travis-ci.org/bjoeris/rust-immutable-seq.svg?branch=master)](https://travis-ci.org/bjoeris/rust-immutable-seq) [![](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/saurvs/astro-rust/blob/master/LICENSE.md)

**Contents**

[API Docs](https://bjoeris.github.io/rust-immutable-seq/)

* [About](#about)
* [Usage](#usage)

## About

`immutable-seq-rust` is a library providing an immutable sequence data structure for the Rust programming language.

The `Seq` implements an API similar to `Vec`, with the added advantage that previous versions of the data structure remain available and unchanged. 

## Usage

* Add the dependency `immutable-seq` to your `Cargo.toml`
  ```toml
  [dependencies]
  immutable-seq = "0.1.0"
  ```

* Include the crate `immutable-seq` in your code
  ```rust
  #[macro_use]
  extern crate immutable_seq;
  
  use immutable_seq::Seq;
  ```
  *(`#[macro_use]` is only required to enable the seq! macro, shown below.)*
  
## Examples

* Create a sequence with some values
  ```rust
  let seq1 : Seq<i32> = seq![1, 2, 3];
  ```
  
* Add an element to the beginninng. *Note:* this creates a *new* sequence, with the element added, but does not change the original sequence.
  ```rust
  let seq2 = seq1.push_front(0);
  assert_eq!(seq1, seq![1, 2, 3]);
  assert_eq!(seq2, seq![0, 1, 2, 3]);
  ```
