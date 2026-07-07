use std::{path::Path, process::Command};

fn main() {
    println!("cargo::rerun-if-changed=frontend");

    let frontend = Path::new("frontend");

    if frontend.join("dist/index.html").exists() {
        return;
    }

    if !has_npm() {
        println!("cargo::error=npm command not found");
        return;
    }

    if !frontend.join("node_modules").exists() {
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
