//! A implementation of the PGM based u64 key indexes described in https://pgm.di.unipi.it/docs/cpp-reference/
//! in rust with zero copy serialization support. Currently only little endian architectures are supported.

// serialization methods is not designed to support big endian systems.
#[cfg(not(target_endian = "little"))]
compile_error!("This program only supports little-endian systems.");

pub mod pgm;
