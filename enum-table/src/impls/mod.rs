mod iter;

mod core;

#[cfg(feature = "std")]
mod map;

#[cfg(feature = "alloc")]
mod vec;
#[cfg(feature = "alloc")]
pub use vec::*;

#[cfg(all(feature = "serde", feature = "alloc"))]
mod serde;
#[cfg(all(feature = "serde", not(feature = "alloc")))]
compile_error!("`serde` feature requires `alloc` feature");
