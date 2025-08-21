#[cfg(feature = "std")]
mod std;

#[cfg(feature = "std")]
mod map;

#[cfg(feature = "alloc")]
mod vec;
#[cfg(feature = "alloc")]
pub use vec::*;

#[cfg(feature = "serde")]
mod serde;
