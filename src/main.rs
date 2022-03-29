// Copyright (c) 2022 myl7
// SPDX-License-Identifier: Apache-2.0

mod cfg;

use crate::cfg::parse_args;
#[allow(unused_imports)]
use byteorder::{BigEndian, LittleEndian, WriteBytesExt};
use log;
use nix::errno::Errno;
use nix::libc;
use nix::sys::ptrace;
use nix::sys::wait::{wait, WaitStatus};
use nix::unistd::{fchdir, Pid};
use path_clean::PathClean;
use simple_logger::SimpleLogger;
use spawn_ptrace::CommandPtraceSpawn;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::from_utf8;
use std::{env, process};

fn main() {
    let cfg = parse_args();

    SimpleLogger::new()
        .with_level(cfg.log_level)
        .init()
        .unwrap();
    std::panic::set_hook(Box::new(|info| {
        log::error!("{}", info);
    }));

    if cfg.cmd.len() <= 0 {
        panic!("No command to be executed");
    }
    let child = Command::new(&cfg.cmd[0])
        .args(&cfg.cmd)
        .spawn_ptrace()
        .expect("Failed to spawn child with ptrace enabled");
    log::debug!("Child started");
    let mut pid = Pid::from_raw(child.id() as libc::pid_t);
    let mut pids = Vec::new();
    ptrace::setoptions(
        pid,
        ptrace::Options::PTRACE_O_EXITKILL | ptrace::Options::PTRACE_O_TRACEFORK,
    )
    .expect("Failed to set child ptrace options");

    let errmsg_ptrace_syscall = "Failed to let child go to next syscall entrance/exit";
    loop {
        match ptrace::syscall(pid, None) {
            Err(errno) => {
                if errno == Errno::ESRCH {
                    break;
                } else {
                    panic!("{}: {:?}", errmsg_ptrace_syscall, errno);
                }
            }
            _ => (),
        }
        let wstatus = wait().unwrap();
        match wstatus {
            WaitStatus::Exited(_, _) | WaitStatus::Signaled(_, _, _) => {
                log::debug!("Process {} exited", pid);
                if pids.is_empty() {
                    break;
                } else {
                    pid = pids.pop().unwrap();
                    continue;
                }
            }
            WaitStatus::PtraceEvent(_, _, event) => {
                if event == libc::PTRACE_EVENT_FORK {
                    pids.push(pid);
                    pid = Pid::from_raw(ptrace::getevent(pid).unwrap() as libc::pid_t);
                    log::debug!("New process {} started", pid);
                    continue;
                }
            }
            _ => (),
        }

        let mut regs = ptrace::getregs(pid).expect("Failed to get child regs at syscall start");
        log::debug!(
            "Trapped child syscall: {}({}, {}, {}, {}, {}, {})",
            regs.orig_rax,
            regs.rdi,
            regs.rsi,
            regs.rdx,
            regs.r10,
            regs.r8,
            regs.r9
        );

        let mut checked = false;
        let mut path = String::new();
        let mut blocked = false;
        match regs.orig_rax as libc::c_long {
            libc::SYS_unlink => {
                checked = true;
                path = get_unlink_path(pid, &regs);
                if path.starts_with("/") {
                    blocked = true;
                    block_syscall(pid, &mut regs);
                }
            }
            libc::SYS_unlinkat => {
                checked = true;
                path = get_unlinkat_path(pid, &regs);
                if path.starts_with("/") {
                    blocked = true;
                    block_syscall(pid, &mut regs);
                }
            }
            _ => (),
        }

        ptrace::syscall(pid, None).expect(errmsg_ptrace_syscall);
        wait().unwrap();

        if checked {
            if blocked {
                regs.rax = (-libc::EPERM) as libc::c_ulonglong;
                ptrace::setregs(pid, regs).expect("Failed to set child regs at syscall end");
                log::info!("The deletion of {} has been blocked", &path);
                break;
            } else {
                log::debug!("The deletion of {} has been allowed", &path)
            }
        }
    }
    log::debug!("Child exited");
}

const ERRMSG_PTRACE_SETREGS_START: &str = "Failed to set child regs at syscall start";

fn block_syscall(pid: Pid, regs: &mut libc::user_regs_struct) {
    regs.orig_rax = u64::MAX as libc::c_ulonglong;
    ptrace::setregs(pid, *regs).expect(ERRMSG_PTRACE_SETREGS_START);
}

const ERRMSG_PTRACE_ARG_STR: &str = "Failed to read path str";

fn get_unlink_path(pid: Pid, regs: &libc::user_regs_struct) -> String {
    let p = ptrace_read_cstr(pid, regs.rdi as *mut libc::c_void).expect(ERRMSG_PTRACE_ARG_STR);
    if !p.starts_with("/") {
        Path::new(&env::current_dir().unwrap())
            .join(&p)
            .clean()
            .to_str()
            .unwrap()
            .to_owned()
    } else {
        PathBuf::from(p).clean().to_str().unwrap().to_owned()
    }
}

fn get_unlinkat_path(pid: Pid, regs: &libc::user_regs_struct) -> String {
    let p = ptrace_read_cstr(pid, regs.rsi as *mut libc::c_void).expect(ERRMSG_PTRACE_ARG_STR);
    if !p.starts_with("/") {
        if regs.rdi as libc::c_int == libc::AT_FDCWD {
            Path::new(&env::current_dir().unwrap())
                .join(&p)
                .clean()
                .to_str()
                .unwrap()
                .to_owned()
        } else {
            let dir_path = get_path_from_dirfd(regs.rdi as libc::c_int);
            Path::new(&dir_path)
                .join(&p)
                .clean()
                .to_str()
                .unwrap()
                .to_owned()
        }
    } else {
        PathBuf::from(p).clean().to_str().unwrap().to_owned()
    }
}

fn get_path_from_dirfd(dirfd: libc::c_int) -> String {
    let (mut reader, writer) = os_pipe::pipe().unwrap();
    let mut cmd = Command::new("true");
    let mut child = unsafe {
        cmd.pre_exec(move || {
            fchdir(dirfd)?;
            let cwd = env::current_dir()?;
            println!("{}", cwd.to_str().unwrap().to_owned());
            process::exit(0);
        })
    }
    .stdout(writer)
    .spawn()
    .unwrap();
    drop(cmd);
    let mut dir_path = String::new();
    reader.read_to_string(&mut dir_path).unwrap();
    child.wait().unwrap();
    if dir_path.len() <= 0
        || dir_path.as_bytes()[0] != '/' as u8
        || dir_path.chars().last().unwrap() != '\n'
    {
        panic!(
            "Failed to get dir path from dirfd: dirfd = {} but path got \"{}\"",
            dirfd, &dir_path
        );
    }
    dir_path
}

fn ptrace_read_cstr(pid: Pid, mut s: *mut libc::c_void) -> nix::Result<String> {
    let mut buf = vec![];
    loop {
        let i = ptrace::read(pid, s)?;
        #[cfg(not(feature = "bigendian"))]
        buf.write_u32::<LittleEndian>(i as u32).unwrap();
        #[cfg(feature = "bigendian")]
        buf.write_u32::<BigEndian>(i as u32).unwrap();
        if buf[buf.len() - 4] == 0 {
            buf.resize(buf.len() - 4, 0);
            break;
        } else if buf[buf.len() - 3] == 0 {
            buf.resize(buf.len() - 3, 0);
            break;
        } else if buf[buf.len() - 2] == 0 {
            buf.resize(buf.len() - 2, 0);
            break;
        } else if buf[buf.len() - 1] == 0 {
            buf.resize(buf.len() - 1, 0);
            break;
        }
        s = unsafe { s.offset(4) };
    }
    Ok(from_utf8(&buf).unwrap().to_owned())
}
