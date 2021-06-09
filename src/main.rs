// main.rs
// :PROPERTIES:
// :header-args: :comments org :tangle src/main.rs
// :END:

use gosh_core::gut::prelude::*;

fn main() -> Result<()> {
    let _ = gosh::repl_enter_main()?;

    Ok(())
}
