// main.rs
// :PROPERTIES:
// :header-args: :comments org :tangle src/main.rs
// :END:
// #+name: 4881dc13

use gosh_core::gut::prelude::*;

fn main() -> Result<()> {
    let _ = gosh::cli::repl_enter_main()?;

    Ok(())
}
