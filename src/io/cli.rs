use std::path::{Path};
use clap::{Parser};

use crate::{Error, Result};

/// Strip the directory and file extension from a file path.
fn file_stem(path: &str) -> Result<&str> {
    let s = Path::new(path).file_stem().ok_or(Error("Empty filename"))?;
    let s = s.to_str().ok_or(Error("Invalid unicode"))?;
    Ok(s)
}

/// Constructs a default output path from `in_path` and `program_name`.
///
/// - in_path - the input path.
/// - program_name - the name of the program.
pub fn default_out_path(in_path: &str, program_name: &str) -> Result<String> {
    let mut out_path = std::env::temp_dir();
    out_path.push(format!("{}-{}.png", file_stem(in_path)?, program_name));
    Ok(out_path.to_str().ok_or(Error("Invalid unicode"))?.to_owned())
}

// ----------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(about = "Process an image file.")]
#[command(author, version, long_about = None)]
pub struct InOutOrder {
    /// Input path.
    pub in_path: String,

    /// Output path.
    #[arg(short, long)]
    pub out_path: Option<String>,

    /// The order of the wavelet pyramid.
    #[arg(short = 'n', long)]
    pub order: Option<usize>,
}

impl InOutOrder {
    /// Returns `out_path` or `default_out_path(program_name)`.
    pub fn out_path(&self, program_name: &str) -> Result<String> {
        self.out_path.clone().map_or_else(|| default_out_path(&self.in_path, program_name), Ok)
    }

    /// Returns the `order` or the specified default value.
    pub fn order(&self, default_order: usize) -> usize {
        self.order.unwrap_or(default_order)
    }
}
