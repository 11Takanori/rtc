extern crate libc;
extern crate nix;

use std::fs;
use nix::sched::*;
use nix::unistd::*;
use nix::unistd::{execv, fork, ForkResult};
use nix::sys::wait::*;
use nix::mount::{mount, MsFlags};
use std::ffi::CString;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    unshare(CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWNS).unwrap_or_else(|e| {
        println!("{:?}", e);
    });

    mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_PRIVATE,
        None::<&str>,
    ).unwrap_or_else(|e| println!("{}", e));
    
    mount(
        Some("/var/lib/test_container/jessie-box"),
        "/var/lib/test_container/jessie-box",
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REC,
        None::<&str>,
    ).unwrap_or_else(|e| {
        println!("{:?}", e);
    });

    chroot("/var/lib/test_container/jessie-box").unwrap_or_else(|e| println!("{}", e));

    chdir("/").unwrap_or_else(|e| println!("{}", e));

    match fork() {
        Ok(ForkResult::Parent { child, .. }) => {
            match waitpid(child, None).expect("wait_pid failed") {
                WaitStatus::Exited(pid, status) => {
                    println!("exit!: pid={:?}, status={:?}", pid, status)
                }
                WaitStatus::Signaled(pid, status, _) => {
                    println!("signal!: pid={:?}, status={:?}", pid, status)
                }
                _ => println!("abnormal exit!"),
            }
        }
        Ok(ForkResult::Child) => {
            sethostname("test_container").expect("sethostname failed.");

            fs::create_dir_all("proc").unwrap_or_else(|why| {
                println!("{:?}", why.kind());
            });

            mount(
                Some("proc"),
                "/proc",
                Some("proc"),
                MsFlags::MS_MGC_VAL,
                None::<&str>,
            ).unwrap_or_else(|e| {
                println!("{:?}", e);
            });

            let dir = CString::new("/bin/bash".to_string()).unwrap();
            let arg = CString::new("-l".to_string()).unwrap();

            execv(&dir, &[dir.clone(), arg]).expect("execution failed.");

            println!("{}", getpid().to_string());
        }
        Err(_) => println!("Fork failed"),
    }
}