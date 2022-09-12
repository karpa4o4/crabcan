use nix::sched::{clone, CloneFlags};
use nix::sys::signal::Signal;
use nix::unistd::{close, execve, Pid};
use std::ffi::CString;
use libc::recv;

use crate::capabilities::setcapabilities;
use crate::config::ContainerOpts;
use crate::errors::Errcode;
use crate::hostname::set_container_hostname;
use crate::mounts::setmountpint;
use crate::namespaces::userns;
use crate::syscalls::setsyscalls;

fn child(config: ContainerOpts) -> isize {
    match setup_container_configuration(&config) {
        Ok(_) => log::info!("Container setup successfully"),
        Err(e) => {
            log::error!("Error while configuring container: {:?}", e);
            return -1;
        }
    }

    if let Err(_) = close(config.fd) {
        log::error!("Error while closing socket");
        return -1;
    }

    log::info!(
        "Starting container with command {} and args {:?}",
        config.path.to_str().unwrap(),
        config.argv,
    );

    let retcode = match execve::<CString, CString>(&config.path, &config.argv, &[]) {
        Ok(_) => 0,
        Err(e) => {
            log::error!("Error while trying to perform execve: {:?}", e);
            -1
        }
    };
    retcode
}

const STACK_SIZE: usize = 1024 * 1024;

pub fn generate_child_process(config: ContainerOpts) -> Result<Pid, Errcode> {
    let mut tmp_stack: [u8; STACK_SIZE] = [0; STACK_SIZE];
    let mut flags = CloneFlags::empty();
    flags.insert(CloneFlags::CLONE_NEWNS);
    flags.insert(CloneFlags::CLONE_NEWCGROUP);
    flags.insert(CloneFlags::CLONE_NEWPID);
    flags.insert(CloneFlags::CLONE_NEWIPC);
    flags.insert(CloneFlags::CLONE_NEWNET);
    flags.insert(CloneFlags::CLONE_NEWUTS);

    match clone(
        Box::new(|| child(config.clone())),
        &mut tmp_stack,
        flags,
        Some(Signal::SIGCHLD as i32),
    ) {
        Ok(pid) => Ok(pid),
        Err(_) => Err(Errcode::ChildProcessError(0)),
    }
}

fn setup_container_configuration(config: &ContainerOpts) -> Result<(), Errcode> {
    setmountpint(&config.mount_dir, &config.addpaths)?;
    set_container_hostname(&config.hostname)?;
    userns(config.fd, config.uid)?;
    setcapabilities()?;
    setsyscalls()?;
    Ok(())
}
