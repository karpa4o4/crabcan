use nix::mount::{MntFlags, mount, MsFlags, umount2};
use rand::Rng;
use std::fs::{create_dir_all, remove_dir};
use std::path::PathBuf;
use nix::unistd::{chdir, pivot_root};

use crate::errors::Errcode;

pub fn setmountpint(mount_dir: &PathBuf, addpaths: &Vec<(PathBuf, PathBuf)>) -> Result<(), Errcode> {
    log::debug!("Setting mount points ...");
    mount_directory(None, &PathBuf::from("/"), vec![MsFlags::MS_REC, MsFlags::MS_PRIVATE])?;

    let new_root = PathBuf::from(format!("/tmp/crabcan.{}", random_string(12)));
    log::debug!("Mounting temp directory {}", new_root.as_path().to_str().unwrap());
    create_directory(&new_root)?;
    mount_directory(Some(mount_dir), &new_root, vec![MsFlags::MS_BIND, MsFlags::MS_PRIVATE])?;

    log::debug!("Mounting additional paths");
    for (inpath, mntpath) in addpaths {
        let outpath = new_root.join(mntpath);
        create_directory(&outpath)?;
        mount_directory(Some(&inpath), &outpath, vec![MsFlags::MS_BIND, MsFlags::MS_PRIVATE])?;
    }

    log::debug!("Pivoting root");
    let old_root_tail = format!("oldroot.{}", random_string(6));
    let put_old = new_root.join(PathBuf::from(old_root_tail.clone()));
    create_directory(&put_old)?;
    if let Err(_) = pivot_root(&new_root, &put_old) {
        return Err(Errcode::MountError(4));
    }

    log::debug!("Unmounting old root");
    let old_root = PathBuf::from(format!("/{}", old_root_tail));

    if let Err(_) = chdir(&PathBuf::from("/")) {
        return Err(Errcode::MountError(5));
    }
    unmount_path(&old_root)?;
    delete_dir(&old_root)?;

    Ok(())
}

fn unmount_path(path: &PathBuf) -> Result<(), Errcode> {
    match umount2(path, MntFlags::MNT_DETACH) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Unable to umount {}: {}", path.as_path().to_str().unwrap(), e);
            Err(Errcode::MountError(0))
        }
    }
}

fn delete_dir(path: &PathBuf) -> Result<(), Errcode> {
    match remove_dir(path) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Unable to delete directory {}: {}", path.as_path().to_str().unwrap(), e);
            Err(Errcode::MountError(1))
        }
    }
}

pub fn clean_mounts(_rootpath: &PathBuf) -> Result<(), Errcode> {
    Ok(())
}

fn mount_directory(path: Option<&PathBuf>, mount_point: &PathBuf, flags: Vec<MsFlags>) -> Result<(), Errcode> {
    let mut ms_flags = MsFlags::empty();
    for f in flags.iter() {
        ms_flags.insert(*f);
    }

    match mount::<PathBuf, PathBuf, PathBuf, PathBuf>(path, mount_point, None, ms_flags, None) {
        Ok(_) => Ok(()),
        Err(e) => {
            if let Some(p) = path {
                log::error!(
                    "Cannot mount {} to {}: {}",
                    p.to_str().unwrap(),
                    mount_point.to_str().unwrap(),
                    e,
                );
            } else {
                log::error!("Cannot remount {}: {}", mount_point.to_str().unwrap(), e);
            }
            Err(Errcode::MountError(3))
        },
    }
}

fn random_string(n: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();
    let name: String = (0..n)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    name
}

fn create_directory(path: &PathBuf) -> Result<(), Errcode> {
    match create_dir_all(path) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Cannot create directory {}: {}", path.to_str().unwrap(), e);
            Err(Errcode::MountError(2))
        }
    }
}
