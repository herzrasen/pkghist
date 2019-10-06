use clap::Shell;

include!("src/opt/cli.rs");

fn main() {
    let outdir = "completions";
    let mut app = build_cli();
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, &outdir);
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::Fish, &outdir);
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::Zsh, &outdir)
}
