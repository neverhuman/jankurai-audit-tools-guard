//! The single `fuser::Filesystem` impl for [`GuardFs`]. This file contains the
//! complete trait impl but delegates the read-path logic to helpers in
//! [`super::ops_read`] and the write/mutation logic to helpers defined here,
//! so neither file grows past the module-size budget.
//!
//! This module is Linux + `fuse`-feature only and is verified on Linux CI.

#![cfg(all(feature = "fuse", target_os = "linux"))]

use super::filesystem::GuardFs;
use super::handles::OpenHandle;
use crate::transaction::{CommitMachine, FsEvent};
use fuser::{
    Errno, FileHandle, Filesystem, FopenFlags, Generation, INodeNo, LockOwner, OpenFlags,
    RenameFlags, ReplyCreate, ReplyEmpty, ReplyWrite, Request, WriteFlags,
};
use std::ffi::OsStr;
use std::path::Path;

// ── write/mutation helpers ────────────────────────────────────────────────────

impl GuardFs {
    /// Feeds write data into the open handle's buffer.
    fn do_write(&self, fh: u64, offset: u64, data: &[u8], reply: ReplyWrite) {
        let mut inner = self.inner.lock().expect("guard fs mutex");
        match inner.handles.get_mut(fh) {
            Some(OpenHandle::Write { machine, .. }) => {
                machine.feed(FsEvent::Write {
                    off: offset,
                    data: data.to_vec(),
                });
                reply.written(data.len() as u32);
            }
            _ => reply.error(Errno::EBADF),
        }
    }

    /// Registers a new write handle for a newly created file.
    fn do_create(&self, parent: u64, name: &OsStr, reply: ReplyCreate) {
        use super::filesystem::TTL;
        let mut inner = self.inner.lock().expect("guard fs mutex");
        let parent_rel = match inner.inodes.path_for(parent) {
            Some(rel) => rel.to_path_buf(),
            None => {
                reply.error(Errno::ENOENT);
                return;
            }
        };
        let rel = parent_rel.join(name);
        let ino = inner.inodes.lookup(&rel);
        let machine = CommitMachine::new_file(rel.clone());
        let fh = inner.handles.insert(OpenHandle::Write { machine });
        reply.created(
            &TTL,
            &GuardFs::overlay_attr(ino, 0),
            Generation(0),
            FileHandle(fh),
            FopenFlags::empty(),
        );
    }

    /// Runs an unlink through the commit machine.
    fn do_unlink(&self, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let mut inner = self.inner.lock().expect("guard fs mutex");
        let parent_rel = match inner.inodes.path_for(parent) {
            Some(rel) => rel.to_path_buf(),
            None => {
                reply.error(Errno::ENOENT);
                return;
            }
        };
        let rel = parent_rel.join(name);
        let base = self.read_or_empty(&rel);
        let mut machine = CommitMachine::existing_file(rel.clone(), base);
        let boundary = machine.feed(FsEvent::Unlink);
        let errno = self.process_boundary(&mut inner, boundary);
        if errno == 0 {
            let _ = std::fs::remove_file(self.backing_path(&rel));
            reply.ok();
        } else {
            reply.error(Errno::from_i32(errno));
        }
    }

    /// Runs a rename through the commit machine.
    fn do_rename(
        &self,
        parent: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        reply: ReplyEmpty,
    ) {
        let mut inner = self.inner.lock().expect("guard fs mutex");
        let from_parent = inner.inodes.path_for(parent).map(Path::to_path_buf);
        let to_parent = inner.inodes.path_for(newparent).map(Path::to_path_buf);
        let (from_parent, to_parent) = match (from_parent, to_parent) {
            (Some(a), Some(b)) => (a, b),
            _ => {
                reply.error(Errno::ENOENT);
                return;
            }
        };
        let from = from_parent.join(name);
        let to = to_parent.join(newname);
        let base = self.read_or_empty(&from);
        let mut machine = CommitMachine::existing_file(from.clone(), base);
        let boundary = machine.feed(FsEvent::Rename {
            from: from.clone(),
            to: to.clone(),
        });
        let errno = self.process_boundary(&mut inner, boundary);
        if errno == 0 {
            let _ = std::fs::remove_file(self.backing_path(&from));
            if let Some(ino) = inner.inodes.ino_for(&from) {
                inner.inodes.rebind(ino, &to);
            }
            reply.ok();
        } else {
            reply.error(Errno::from_i32(errno));
        }
    }
}

// ── Filesystem trait impl ─────────────────────────────────────────────────────
// Read-path handlers delegate to helpers defined in ops_read.rs.
// Write/mutation handlers delegate to helpers defined above.

impl Filesystem for GuardFs {
    // ── read-path ────────────────────────────────────────────────────────────

    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: fuser::ReplyEntry) {
        self.handle_lookup(parent.0, name, reply);
    }

    fn forget(&self, _req: &Request, ino: INodeNo, nlookup: u64) {
        self.handle_forget(ino.0, nlookup);
    }

    fn getattr(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: Option<FileHandle>,
        reply: fuser::ReplyAttr,
    ) {
        self.handle_getattr(ino.0, reply);
    }

    fn open(&self, _req: &Request, ino: INodeNo, flags: OpenFlags, reply: fuser::ReplyOpen) {
        self.handle_open(ino.0, flags.0, reply);
    }

    fn read(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        size: u32,
        _flags: OpenFlags,
        _lock: Option<LockOwner>,
        reply: fuser::ReplyData,
    ) {
        self.handle_read(ino.0, offset, size, reply);
    }

    fn readdir(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        reply: fuser::ReplyDirectory,
    ) {
        self.handle_readdir(ino.0, offset, reply);
    }

    fn readlink(&self, _req: &Request, ino: INodeNo, reply: fuser::ReplyData) {
        self.handle_readlink(ino.0, reply);
    }

    // ── write/mutation ────────────────────────────────────────────────────────

    fn write(
        &self,
        _req: &Request,
        _ino: INodeNo,
        fh: FileHandle,
        offset: u64,
        data: &[u8],
        _write_flags: WriteFlags,
        _flags: OpenFlags,
        _lock: Option<LockOwner>,
        reply: ReplyWrite,
    ) {
        self.do_write(fh.0, offset, data, reply);
    }

    fn fsync(&self, _req: &Request, _ino: INodeNo, fh: FileHandle, _ds: bool, reply: ReplyEmpty) {
        self.commit_handle(fh.0, FsEvent::Fsync, reply);
    }

    fn flush(
        &self,
        _req: &Request,
        _ino: INodeNo,
        fh: FileHandle,
        _lo: LockOwner,
        reply: ReplyEmpty,
    ) {
        self.commit_handle(fh.0, FsEvent::Flush, reply);
    }

    fn release(
        &self,
        _req: &Request,
        _ino: INodeNo,
        fh: FileHandle,
        _flags: OpenFlags,
        _lock_owner: Option<LockOwner>,
        _flush: bool,
        reply: ReplyEmpty,
    ) {
        let mut inner = self.inner.lock().expect("guard fs mutex");
        let handle = inner.handles.remove(fh.0);
        match handle {
            Some(OpenHandle::Write { mut machine, .. }) => {
                let boundary = machine.feed(FsEvent::Release);
                let errno = self.process_boundary(&mut inner, boundary);
                if errno == 0 {
                    reply.ok();
                } else {
                    reply.error(Errno::from_i32(errno));
                }
            }
            _ => reply.ok(),
        }
    }

    fn create(
        &self,
        _req: &Request,
        parent: INodeNo,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        _flags: i32,
        reply: ReplyCreate,
    ) {
        self.do_create(parent.0, name, reply);
    }

    fn unlink(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
        self.do_unlink(parent.0, name, reply);
    }

    fn rename(
        &self,
        _req: &Request,
        parent: INodeNo,
        name: &OsStr,
        newparent: INodeNo,
        newname: &OsStr,
        _flags: RenameFlags,
        reply: ReplyEmpty,
    ) {
        self.do_rename(parent.0, name, newparent.0, newname, reply);
    }
}
