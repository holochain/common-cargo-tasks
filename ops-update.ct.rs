/*
@ct-help@ Install/Update cargo-task and other ops utils. @@
*/

use std::path::Path;

mod cargo_task_util;
use cargo_task_util::*;

/// entrypoint
fn main() {
    check_cargo_task_dir();
    check_git_ignore();
    check_ci_tasks();
}

/// Execute a git command.
fn git<R: AsRef<std::ffi::OsStr>>(args: impl IntoIterator<Item = R>) -> bool {
    let mut cmd = std::process::Command::new("git");
    cmd.args(args);
    cmd.spawn().unwrap().wait().unwrap().success()
}

/// see if the .cargo-task directory can be updated
fn check_cargo_task_dir() {
    ct_info!("checking root .cargo-task");

    // get the current checkout commit hash
    let mut cmd = std::process::Command::new("git");
    let cur = cmd
        .args(&["-C", ".cargo-task", "rev-parse", "HEAD"])
        .output()
        .unwrap();
    let cur = String::from_utf8_lossy(&cur.stdout);
    let cur = cur.trim();

    let mut cmd = std::process::Command::new("git");
    let rev = cmd
        .args(&["ls-remote", "https://github.com/holochain/common-cargo-tasks.git", "main"])
        .output()
        .unwrap();
    let rev = String::from_utf8_lossy(&rev.stdout);
    let mut rev = rev.split_whitespace();
    let rev = rev.next().unwrap();
    let rev = rev.trim();

    if rev == cur {
        ct_info!(".cargo-task commit hash is up to date");
    } else {
        ct_info!("updating .cargo-task commit hash to {}", rev);

        git(&["-C", ".cargo-task", "reset", "--hard"]);
        git(&["-C", ".cargo-task", "fetch", "origin", &rev]);
        git(&["-C", ".cargo-task", "checkout", &rev]);

        std::fs::write(".cargo-task/.git-pin", rev).unwrap();

        ct_info!(".cargo-task updated - re-running `cargo task ops-update`");
        let mut cmd = ct_env().cargo();
        cmd
            .arg("task")
            .arg("ops-update");
        ct_check_fatal!(ct_env().exec(cmd));
        std::process::exit(0);
    }
}

/// ensure we have certain directives in the root .gitignore
fn check_git_ignore() {
    ct_info!("checking root .gitignore");

    const IGNORE: &str = ".cargo-task/*";
    const EX_BOOTSTRAP: &str = "!.cargo-task/bootstrap.ct.rs";
    const EX_PIN: &str = "!.cargo-task/.git-pin";

    let mut has_ignore = false;
    let mut has_ex_bootstrap = false;
    let mut has_ex_pin = false;
    let mut has_trailing_newline = false;

    if let Ok(data) = std::fs::read(".gitignore") {
        if data[data.len() - 1] == 10 || data[data.len() - 1] == 13 {
            has_trailing_newline = true;
        }
        let data = String::from_utf8_lossy(&data);
        for line in data.split_whitespace() {
            if line == IGNORE {
                has_ignore = true;
            }
            if line == EX_BOOTSTRAP {
                has_ex_bootstrap = true;
            }
            if line == EX_PIN {
                has_ex_pin = true;
            }
        }
    }

    let mut f = ct_check_fatal!(std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(".gitignore"));

    use std::io::Write;
    if !has_trailing_newline {
        ct_check_fatal!(f.write_all(b"\n"));
    }
    if !has_ignore {
        ct_info!("root .gitignore append '{}'", IGNORE);
        ct_check_fatal!(f.write_all(IGNORE.as_bytes()));
    }
    if !has_ex_bootstrap {
        ct_info!("root .gitignore append '{}'", EX_BOOTSTRAP);
        ct_check_fatal!(f.write_all(EX_BOOTSTRAP.as_bytes()));
    }
    if !has_ex_pin {
        ct_info!("root .gitignore append '{}'", EX_PIN);
        ct_check_fatal!(f.write_all(EX_PIN.as_bytes()));
    }
}

/// ensure we have certain github actions configured
fn check_ci_tasks() {
    ct_info!("checking root .github");

    copy_dir(".cargo-task/.resources/.github", ".github");
}

/// recursively copy a whole directory
fn copy_dir<S: AsRef<Path>, D: AsRef<Path>>(src: S, dest: D) {
    ct_check_fatal!(std::fs::create_dir_all(&dest));
    for item in ct_check_fatal!(std::fs::read_dir(src)) {
        if let Ok(item) = item {
            let meta = ct_check_fatal!(item.metadata());
            let mut dest = dest.as_ref().to_owned();
            dest.push(item.file_name());
            if meta.is_dir() {
                copy_dir(item.path(), &dest);
            } else if meta.is_file() {
                ct_check_fatal!(std::fs::copy(item.path(), &dest));
            }
        }
    }
}
