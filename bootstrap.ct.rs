//! We will only check this script and the `.git-pin` file into HC rust repos
//! On initial `cargo task` run on new checkouts, the `.cargo-task`
//! will be fetched as a shallow git clone.
//! The `.git-pin` will point to the pinned git commit hash.

/*
@ct-bootstrap@ true @@

@ct-min-version@ 0.0.10 @@

@ct-help@ Checkout/update the Holo common .cargo-task tasks. @@
*/

mod cargo_task_util;

/// Execute a git command.
fn git<R: AsRef<std::ffi::OsStr>>(args: impl IntoIterator<Item = R>) -> bool {
    let mut cmd = std::process::Command::new("git");
    cmd.args(args);
    cmd.spawn().unwrap().wait().unwrap().success()
}

/// Entrypoint.
fn main() {
    // If .cargo-task is not a git repository,
    // Remove it and check it out
    if !std::path::Path::new(".cargo-task/.git").exists() {
        // shallow git clone
        if !git(&[
            "clone",
            "file:///home/neonphog/tmp/hc-cargo-task",
            "--depth=1",
            "--branch",
            "main",
            "--single-branch",
            ".cargo-task-tmp",
        ]) {
            ct_fatal!("failed to execute 'git' command");
        }

        // move the .git-pin file, if set
        let _ = std::fs::rename(".cargo-task/.git-pin", ".cargo-task-tmp/.git-pin");

        // if the checkout succeeded, move the directory in
        let _ = std::fs::remove_dir_all(".cargo-task");
        let _ = std::fs::rename(".cargo-task-tmp", ".cargo-task");
    }

    // get the current checkout commit hash
    let mut cmd = std::process::Command::new("git");
    let rev = cmd
        .args(&["-C", ".cargo-task", "rev-parse", "HEAD"])
        .output()
        .unwrap();
    let rev = String::from_utf8_lossy(&rev.stdout);
    let rev = rev.trim();

    if let Ok(pin) = std::fs::read("./.cargo-task/.git-pin") {
        // if we have a pin file - ensure we check that commit hash out
        let pin = String::from_utf8_lossy(&pin);
        let pin = pin.trim();
        if rev != pin {
            // reset any local changes
            git(&["-C", ".cargo-task", "reset", "--hard"]);
            // fetch that hash (remember we are a shallow clone)
            git(&["-C", ".cargo-task", "fetch", "origin", &pin]);
            // switch to that commit hash (detached head)
            git(&["-C", ".cargo-task", "checkout", &pin]);
        }
    } else {
        // if we don't have a pin file - create one at the current hash
        std::fs::write(".cargo-task/.git-pin", rev).unwrap();
    }
}
