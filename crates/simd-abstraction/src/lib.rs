#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![cfg_attr(feature = "unstable", feature(stdsimd))]
#![cfg_attr(feature = "unstable", feature(arm_target_feature))]
#![cfg_attr(docsrs, feature(doc_cfg))]
//
#![deny(
    missing_debug_implementations,
    clippy::all,
    clippy::cargo,
    clippy::missing_inline_in_public_items
)]
#![warn(clippy::todo)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
mod macros;

pub mod tools;

pub mod traits;

pub mod arch {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub mod x86;

    #[cfg(all(feature = "unstable", target_arch = "arm"))]
    pub mod arm;

    #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
    pub mod aarch64;

    #[cfg(target_arch = "wasm32")]
    pub mod wasm;
}

pub mod common {
    pub mod ascii;
    pub mod crc32;
    pub mod hex;
}

pub use outref::{OutBuf, OutRef};
