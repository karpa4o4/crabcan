use std::fmt;
use std::fmt::{Formatter};
use std::process::exit;

#[derive(Debug)]
// Contains all possible errors in our tools.
pub enum Errcode {
    ArgumentInvalid(&'static str),
    CapabilitiesError(u8),
    ChildProcessError(u8),
    ContainerError(u8),
    HostnameError(u8),
    MountError(u8),
    NamespaceError(u8),
    NotSupported(u8),
    ResourcesError(u8),
    RngError,
    SocketError(u8),
    SyscallsError(u8),
}

impl Errcode {
    pub fn get_retcode(&self) -> i32 {
        1
    }
}

#[allow(unreachable_patterns)]
impl fmt::Display for Errcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            Errcode::ArgumentInvalid(element) => write!(f, "ArgumentInvalid: {}", element),
            _ => write!(f, "{:?}", self),
        }
    }
}

pub fn exit_with_retcode(res: Result<(), Errcode>) {
    match res {
        Ok(_) => {
            log::info!("Exit without any error, returning 0.");
            exit(0);
        },
        Err(e) => {
            let retcode = e.get_retcode();
            log::error!("Error on exit:\n\t{}\n\tReturning {}", e, retcode);
            exit(retcode);
        },
    }
}