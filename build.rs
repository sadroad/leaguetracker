use std::process::Command;

fn main() {
    let path = std::fs::canonicalize("./frontend").unwrap();
    Command::new("pnpm")
        .args(["build"])
        .current_dir(path)
        .status()
        .expect("Failed to build site");
}
