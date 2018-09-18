// mod.rs
// :PROPERTIES:
// :header-args: :comments org :tangle src/adaptors/mod.rs
// :END:

use std::path::Path;
use models::*;
use quicli::prelude::*;

use gchemol::io;

pub trait ModelAdaptor {
    /// Parse calculated properties from output file.
    fn parse_outfile<P: AsRef<Path>>(&self, outfile: P) -> Result<ModelProperties> {
        let outfile = outfile.as_ref();
        let output = io::read_file(outfile)?;
        self.parse_stream(&output)
    }

    /// Parse calculated properties from text stream.
    fn parse_stream(&self, output: &str) -> Result<ModelProperties>;
}

mod mopac;
pub use self::mopac::MOPAC;
