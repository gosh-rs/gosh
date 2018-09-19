// mod.rs
// :PROPERTIES:
// :header-args: :comments org :tangle src/adaptors/mod.rs
// :END:

use std::path::Path;
use std::str::Lines;
use models::*;
use quicli::prelude::*;

use gchemol::io;

pub trait ModelAdaptor {
    /// Parse calculated properties from output file.
    fn parse_outfile<P: AsRef<Path>>(&self, outfile: P) -> Result<ModelProperties> {
        let outfile = outfile.as_ref();
        let output = io::read_file(outfile)?;
        self.parse_one(&output)
    }

    /// Parse a single entry of ModelProperties from line oriented iterator
    fn parse_one(&self, output: &str) -> Result<ModelProperties> {
        let mlist = self.parse_all(output)?;
        let n = mlist.len();

        // take the last part
        if n > 0 {
            Ok(mlist[n-1].clone())
        } else {
            bail!("got nothing!");
        }
    }

    /// Parse all properties in multi-step calculation, sush as optimization or
    /// multi-molecule batch calculation.
    ///
    /// Return a list of ModelProperties
    fn parse_all(&self, output: &str) -> Result<Vec<ModelProperties>>;
}

mod mopac;
pub use self::mopac::MOPAC;
