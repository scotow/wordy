use std::{path::Path, process::Command};

fn main() {
    let frontend = Path::new("frontend");

    println!("cargo::rerun-if-changed=frontend");

    if !has_npm() && !Path::new("frontend/dist/index.html").exists() {
        println!("cargo::error=npm command and dist asset not found");
        return;
    }

    if !Path::new("frontend/node_modules").exists() {
        run(
            Command::new("npm").arg("install").current_dir(frontend),
            "npm install",
        );
    }

    run(
        Command::new("npm")
            .args(["run", "build"])
            .current_dir(frontend),
        "npm run build",
    );
}

fn has_npm() -> bool {
    Command::new("npm")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn run(cmd: &mut Command, label: &str) {
    let status = cmd
        .status()
        .unwrap_or_else(|e| panic!("failed to spawn `{label}`: {e}"));
    if !status.success() {
        panic!("`{label}` failed with status {status}");
    }
}
