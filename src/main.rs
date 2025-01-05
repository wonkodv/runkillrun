use std::env;
use std::path::Path;
use std::process::{exit, Command};
use std::time::{Duration, SystemTime};

use anyhow::Result;
use anyhow::{bail, Context};

fn main() {
    match try_main() {
        Ok(Some(rc)) => exit(rc),
        Ok(None) => exit(43),
        Err(message) => {
            eprintln!("[RKR] {message}");
            eprintln!("[RKR] Usage: rkr EXE [ARG *]");
            eprintln!("[RKR] -   EXE must be a (relative/absolute) path to an executable file");
            eprintln!("[RKR] -   rkr will spawn EXE with ARGS. Whenever EXE changes, the target is killed and started again");
            eprintln!("[RKR] -   rkr exits with:");
            eprintln!("[RKR]     -   42 on error");
            eprintln!("[RKR]     -   43 if the target exited due to a signal");
            eprintln!("[RKR]     -   return code of the target process if target exits normally");
            eprintln!("[RKR] -   rkr only prints to stderr, each line is prefixed with `[RKR] `");
            exit(42);
        }
    };
}

fn get_meta(path: &Path) -> Result<(u64, SystemTime, SystemTime)> {
    let metadata =
        std::fs::metadata(path).with_context(|| format!("Can't get MetaData for {path:?}"))?;
    Ok((
        metadata.len(),
        metadata
            .modified()
            .with_context(|| format!("Can't get mtime for {path:?}"))?,
        metadata
            .created()
            .with_context(|| format!("Can't get ctime for {path:?}"))?,
    ))
}

fn try_main() -> Result<Option<i32>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        bail!("Not enough Arguments");
    }

    let exe = &args[1];
    let exe_path = Path::new(exe);
    let args = &args[2..];

    loop {
        let meta = get_meta(exe_path)?;
        eprintln!("[RKR] Meta for {exe} {meta:?}");

        eprintln!("[RKR] Spawning {exe} {args:?}");
        let mut child = Command::new(exe)
            .args(args)
            .spawn()
            .with_context(|| format!("Error spawning Process {exe}"))?;

        let child_pid = child.id();
        eprintln!("[RKR] Target has {child_pid}");

        'poll: loop {
            std::thread::sleep(Duration::from_secs(1));
            if let Some(return_code) = child
                .try_wait()
                .context(format!("Waiting on target process {child:?}"))?
            {
                eprintln!("[RKR] Target {child_pid} exited with {return_code}");
                return Ok(return_code.code());
            };

            match get_meta(exe_path) {
                Err(err) => {
                    eprintln!(
                        "[RKR] can not get Meta for {exe}, waiting for file to appear. ({err:?})"
                    );
                }
                Ok(new_meta) => {
                    if new_meta != meta {
                        eprintln!(
                            "[RKR] File Changed: {meta:?} != {new_meta:?}. killing {child_pid}"
                        );
                        child.kill()?;
                        child.wait()?;
                        break 'poll;
                    }
                }
            }
        }
    }
}
