// mod.rs
// :PROPERTIES:
// :header-args: :comments org :tangle src/adaptors/mod.rs
// :END:

use std::io::{Read, BufRead, BufReader};
use std::path::Path;

use quicli::prelude::*;
use nom::{self, IResult};

use gchemol::io;

use crate::models::*;

/// Common interface for model adaptors
pub trait ModelAdaptor {
    /// Parse the last entry of ModelProperties from a calculation output file
    /// # Return
    /// - ModelProperties, the calculated properties, including energy, forces, ...
    fn parse_last<P: AsRef<Path>>(&self, outfile: P) -> Result<ModelProperties>;

    /// Parse all properties in multi-step calculation, sush as optimization or
    /// multi-molecule batch calculation.
    ///
    /// # Return
    /// - a list of ModelProperties
    fn parse_all<P: AsRef<Path>>(&self, outfile: P) -> Result<Vec<ModelProperties>>;
}

// the type for the parsed part
type Part<'a> =  ModelProperties;

mod mopac;
pub use self::mopac::MOPAC;
