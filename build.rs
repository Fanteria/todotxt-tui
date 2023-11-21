// use clap::ValueEnum;
// use clap_complete::{generate_to, Shell};
// use std::env;
use std::io::Error;

// mod error {
//     include!("src/error.rs");
// }
// mod config {
//     // include!("src/config.rs");
//     mod colors {
//         include!("src/config/colors.rs");
//     }
//     mod keycode {
//         include!("src/config/keycode.rs");
//     }
//     // mod logger {
//     //     include!("src/config/logger.rs");
//     // }
//     // mod styles {
//     //     include!("src/config/styles.rs");
//     // }
//     mod text_modifier {
//         include!("src/config/text_modifier.rs");
//     }
//     mod text_style {
//         include!("src/config/text_style.rs");
//     }
// }

fn main() -> Result<(), Error> {
    // let outdir = match env::var_os("OUT_DIR") {
    //     None => return Ok(()),
    //     Some(outdir) => outdir,
    // };

    // let mut cmd = build_cli();
    // for &shell in Shell::value_variants() {
    //     generate_to(shell, &mut cmd, "myapp", outdir)?;
    // }

    Ok(())
}
