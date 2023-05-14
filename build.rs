use std::println;

use clap_complete::generate_to;
use clap_complete::Shell;

include!("src/opt/cli.rs");

fn main() {
    let out_dir = "completions";
    let mut command = build_cli();
    for shell in [Shell::Bash, Shell::Fish, Shell::Zsh] {
        match generate_to(shell, &mut command, env!("CARGO_PKG_NAME"), out_dir) {
            Ok(_) => println!("Successfully generated {} completions", shell.to_string()),
            Err(err) => println!(
                "Unable to generate {} completions {}",
                shell.to_string(),
                err.to_string()
            ),
        };
    }
}
