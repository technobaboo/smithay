//! Abstraction of different session APIs.
//!
//! Sessions provide a way for multiple graphical systems to run in parallel by providing
//! mechanisms to switch between and handle device access and permissions for every running
//! instance. They are crucial to allow unprivileged processes to use graphical or input
//! devices.
//!
//! The functions of the provided interfaces lies in two main components: the ability to
//! open privileged devices, and be notified about when the session is paused and resumed
//! (for example when the user switches to an other TTY, or is the computer goes to sleep).
//!
//! ## General use
//!
//! Session handling in Smithay is done through a pair of types that each session provider implements.
//!
//! The first is a handle implementing the [`Session`] trait, which allows you to request the opening
//! of devices, a VT change, or information about the session state.
//!
//! The second is a notifier which informs you when the session is enabled or disabled by the system.
//! This notifier takes the form of a [`calloop`] event source to deliver pause and activation events.
//!
//! ## Available providers
//!
//! This module provides just one session implementation, through [libseat](https://sr.ht/~kennylevinsen/seatd/),
//! gated by the `backend_session_libseat` cargo feature.
//!
//! Other implementations can be provided out-of-tree.

use nix::fcntl::OFlag;
use std::{
    cell::RefCell,
    os::unix::io::RawFd,
    path::Path,
    rc::Rc,
    sync::{Arc, Mutex},
};

/// General session interface.
///
/// Provides a way to open and close devices and change the active vt.
pub trait Session {
    /// Error type of the implementation
    type Error: AsErrno;

    /// Opens a device at the given `path` with the given flags.
    ///
    /// Returns a raw file descriptor
    fn open(&mut self, path: &Path, flags: OFlag) -> Result<RawFd, Self::Error>;
    /// Close a previously opened file descriptor
    fn close(&mut self, fd: RawFd) -> Result<(), Self::Error>;

    /// Change the currently active virtual terminal
    fn change_vt(&mut self, vt: i32) -> Result<(), Self::Error>;

    /// Check if this session is currently active
    fn is_active(&self) -> bool;
    /// Which seat this session is on
    fn seat(&self) -> String;
}

/// Events that can be generated by a session
#[derive(Copy, Clone, Debug)]
pub enum Event {
    /// The whole session has been paused
    ///
    /// All devices should be considered as paused
    PauseSession,
    /// The whole session has been activated
    ActivateSession,
}

impl Session for () {
    type Error = ();

    fn open(&mut self, _path: &Path, _flags: OFlag) -> Result<RawFd, Self::Error> {
        Err(())
    }
    fn close(&mut self, _fd: RawFd) -> Result<(), Self::Error> {
        Err(())
    }

    fn change_vt(&mut self, _vt: i32) -> Result<(), Self::Error> {
        Err(())
    }

    fn is_active(&self) -> bool {
        false
    }
    fn seat(&self) -> String {
        String::from("seat0")
    }
}

impl<S: Session> Session for Rc<RefCell<S>> {
    type Error = S::Error;

    fn open(&mut self, path: &Path, flags: OFlag) -> Result<RawFd, Self::Error> {
        self.borrow_mut().open(path, flags)
    }

    fn close(&mut self, fd: RawFd) -> Result<(), Self::Error> {
        self.borrow_mut().close(fd)
    }

    fn change_vt(&mut self, vt: i32) -> Result<(), Self::Error> {
        self.borrow_mut().change_vt(vt)
    }

    fn is_active(&self) -> bool {
        self.borrow().is_active()
    }

    fn seat(&self) -> String {
        self.borrow().seat()
    }
}

impl<S: Session> Session for Arc<Mutex<S>> {
    type Error = S::Error;

    fn open(&mut self, path: &Path, flags: OFlag) -> Result<RawFd, Self::Error> {
        self.lock().unwrap().open(path, flags)
    }

    fn close(&mut self, fd: RawFd) -> Result<(), Self::Error> {
        self.lock().unwrap().close(fd)
    }

    fn change_vt(&mut self, vt: i32) -> Result<(), Self::Error> {
        self.lock().unwrap().change_vt(vt)
    }

    fn is_active(&self) -> bool {
        self.lock().unwrap().is_active()
    }

    fn seat(&self) -> String {
        self.lock().unwrap().seat()
    }
}

/// Allows errors to be described by an error number
pub trait AsErrno: ::std::fmt::Debug {
    /// Returns the error number representing this error if any
    fn as_errno(&self) -> Option<i32>;
}

impl AsErrno for () {
    fn as_errno(&self) -> Option<i32> {
        None
    }
}

#[cfg(feature = "backend_session_libseat")]
pub mod libseat;
