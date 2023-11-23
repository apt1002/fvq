/// A catch-all error type.
#[derive(Debug, Copy, Clone)]
pub struct Error(pub &'static str);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

// ----------------------------------------------------------------------------

/// A general `Result` type.
pub type Result<T=()> = std::result::Result<T, Box<dyn std::error::Error>>;

// ----------------------------------------------------------------------------

/// Tile/pixel coordinates, with `(0, 0)` at the top left. The coordinates are
/// listed in the order `(row, column)`, i.e. y-coordinate first.
pub type Grid = (usize, usize);

/// The `Index` type of a 2x2 grid. The coordinates are listed in the order
/// `(row, column)`, i.e. y-coordinate first.
pub type Small = (bool, bool);

// ----------------------------------------------------------------------------

pub mod io;

mod quad;
pub use quad::{Quad, Tree, Branch};

pub mod transform;
pub use transform::{Position, Pyramid, VHC};

pub mod quantize;
