mod std;

mod vec;
pub use vec::*;

#[cfg(feature = "serde")]
mod serde;
