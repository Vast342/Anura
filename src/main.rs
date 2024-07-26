pub mod uci;

use crate::uci::uci_main;

fn main() {
    loop {
        uci_main();
    }
}

// pext is _pext_u64