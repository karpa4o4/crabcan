use std::ffi::CString;
use std::os::unix::io::RawFd;
use std::path::PathBuf;

use crate::errors::Errcode;
use crate::hostname::generate_hostname;
use crate::ipc::generate_socketpair;

#[derive(Clone)]
pub struct ContainerOpts {
    pub path:       CString,
    pub argv:       Vec<CString>,

    pub uid:        u32,
    pub mount_dir:  PathBuf,
    pub addpaths: Vec<(PathBuf, PathBuf)>,

    pub fd:         RawFd,
    pub hostname:   String,
}

impl ContainerOpts {
    pub fn new(command: String, uid: u32, mount_dir: PathBuf, addpaths: Vec<(PathBuf, PathBuf)>) -> Result<(ContainerOpts, (RawFd, RawFd)), Errcode>{
        let argv: Vec<CString> = command.split_ascii_whitespace()
            .map(|s| CString::new(s).expect("Cannot read arg"))
            .collect();
        let path = argv[0].clone();

        let sockets = generate_socketpair()?;

        Ok((
            ContainerOpts {
                path,
                argv,
                uid,
                mount_dir,
                addpaths,
                fd: sockets.1.clone(),
                hostname: generate_hostname()?,
            },
            sockets,
        ))
    }
}