use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::env;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{exit, Command, ExitStatus};
use std::sync::mpsc::channel;

#[derive(Debug)]
enum Message {
    FileWatch(Result<notify::Event, notify::Error>),
    ProcessWatch(Result<ExitStatus, std::io::Error>),
}

fn main() {
    match try_main() {
        Ok(rc) => exit(rc),
        Err(message) => {
            eprintln!("[RKR] {message}");
            eprintln!("[RKR] Usage: rkr EXE [ARG *]");
            eprintln!("[RKR] -   EXE must be a (relative/absolute) path to an executable file");
            eprintln!("[RKR] -   rkr will spawn EXE with ARGS. Whenever EXE changes, the target is killed and started again");
            eprintln!("[RKR] -   rkr exits with:");
            eprintln!("[RKR]     -   42 on error");
            eprintln!("[RKR]     -   43 if the target exited due to a signal");
            eprintln!("[RKR]     -   return code of the target process if target exits normally");
            exit(42);
        }
    };
}

fn try_main() -> Result<i32, String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Not enough arguments".to_owned());
    }

    let exe = &args[1];
    let args = &args[2..];

    match std::fs::metadata(exe) {
        Err(err) => return Err(format!("Can not inspect {exe}: {err}")),
        Ok(stat) => {
            if !stat.is_file() {
                return Err(format!("Not a file: {exe}"));
            }
        }
    }

    let (tx, rx) = channel();

    let mut watcher = {
        let tx = tx.clone();
        notify::recommended_watcher(move |event| {
            tx.send(Message::FileWatch(event)).unwrap();
        })
        .map_err(|err| format!("Can not watch {exe}: {err}"))?
    };
    watcher
        .watch(Path::new(exe), RecursiveMode::NonRecursive)
        .map_err(|err| format!("Can not watch {exe}: {err}"))?;

    'respawn: loop {
        eprintln!("[RKR] Spawning {exe} {args:?}");
        let mut child = Command::new(exe)
            .args(args)
            .spawn()
            .map_err(|err| format!("Error spawning Process: {err}"))?;

        let child_pid = child.id();
        eprintln!("[RKR] Target has {child_pid}");
        {
            let tx = tx.clone();
            std::thread::spawn(move || {
                let wait = child.wait();
                eprintln!("[RKR] Target exited: {wait:?}");
                tx.send(Message::ProcessWatch(wait)).unwrap();
            });
        }
        // inner Loop to not respawn on all the uninteresting file events
        loop {
            match rx.recv().expect("pipes work") {
                Message::FileWatch(Err(err)) => {
                    return Err(format!("File Watch Error: {err}"));
                }
                Message::FileWatch(Ok(Event {
                    kind: EventKind::Modify(_),
                    ..
                }))
                | Message::FileWatch(Ok(Event {
                    kind: EventKind::Create(_),
                    ..
                })) => {
                    eprintln!("[RKR] File Changed, killing {child_pid}");
                    if unsafe { libc::kill(child_pid as i32, libc::SIGINT) } != 0 {
                        return Err(format!("can not kill {child_pid}"));
                    }
                    if let Message::ProcessWatch(Ok(return_code)) = rx.recv().expect("pipes work") {
                        if return_code.signal() == Some(libc::SIGINT) {
                            continue 'respawn;
                        }
                        eprintln!("[RKR] Process exited after we killed it, but with an unexpected signal: {return_code}");
                        exit(return_code.code().unwrap_or(43));
                    }
                }
                Message::FileWatch(Ok(_)) => {}
                Message::ProcessWatch(Err(err)) => {
                    return Err(format!("Process Wait Error: {err}"));
                }
                Message::ProcessWatch(Ok(return_code)) => {
                    eprintln!("[RKR] Process exited: {return_code}");
                    exit(return_code.code().unwrap_or(43));
                }
            }
        }
    }
}
