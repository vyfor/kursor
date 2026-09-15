use std::{
    fs::{File, OpenOptions},
    io::{self, IsTerminal, Read, Write},
    os::fd::AsRawFd,
    time::{Duration, Instant},
};

use kursor_core::term_info::TermInfo;

use super::{
    ProbeParser, ProbeSession, UnixRoute, direct_queries, passthrough_queries,
    unix_route,
};

pub fn query(timeout: Duration, info: &mut TermInfo) -> io::Result<bool> {
    let mut tty = open_tty()?;
    let fd = tty.as_raw_fd();

    let original = unsafe {
        let mut t = std::mem::zeroed();
        if libc::tcgetattr(fd, &mut t) != 0 {
            return Err(io::Error::last_os_error());
        }
        t
    };

    let route = unix_route();
    let mut parser = ProbeParser::default();
    let mut session = ProbeSession::new(info);
    let result =
        exchange(&mut tty, fd, timeout, &mut parser, &mut session, route);

    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &original) } != 0 {
        return Err(io::Error::last_os_error());
    }

    result?;
    Ok(session.route())
}

fn exchange(
    tty: &mut File,
    fd: i32,
    timeout: Duration,
    parser: &mut ProbeParser,
    session: &mut ProbeSession<'_>,
    route: UnixRoute,
) -> io::Result<()> {
    let raw = unsafe {
        let mut t = std::mem::zeroed::<libc::termios>();
        if libc::tcgetattr(fd, &mut t) != 0 {
            return Err(io::Error::last_os_error());
        }
        libc::cfmakeraw(&mut t);
        t
    };
    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } != 0 {
        return Err(io::Error::last_os_error());
    }

    let multiplexer = route != UnixRoute::Direct;
    let direct = direct_queries(session.info(), multiplexer);
    if direct.is_empty() {
        return Ok(());
    }
    tty.write_all(direct.as_bytes())?;
    tty.flush()?;

    let deadline = Instant::now() + timeout;
    let mut buf = [0u8; 4096];

    if !read(tty, fd, deadline, parser, session, &mut buf)? {
        return Ok(());
    }
    if route != UnixRoute::Direct {
        session.passthrough();
        let passthrough = passthrough_queries(session.info());
        tty.write_all(passthrough.as_bytes())?;
        tty.flush()?;
        let _ = read(tty, fd, deadline, parser, session, &mut buf)?;
    }
    Ok(())
}

fn read(
    tty: &mut File,
    fd: i32,
    deadline: Instant,
    parser: &mut ProbeParser,
    session: &mut ProbeSession<'_>,
    buf: &mut [u8; 4096],
) -> io::Result<bool> {
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(false);
        }
        let mut pollfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let ms = remaining.as_millis().min(i32::MAX as u128) as i32;
        let ready = unsafe { libc::poll(&mut pollfd, 1, ms) };
        if ready == 0 {
            return Ok(false);
        }
        if ready < 0 {
            return Err(io::Error::last_os_error());
        }

        let count = tty.read(buf)?;
        if count == 0 {
            return Ok(false);
        }
        if session.accept(parser.feed(&buf[..count])) {
            return Ok(true);
        }
    }
}

fn open_tty() -> io::Result<File> {
    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        OpenOptions::new().read(true).write(true).open("/dev/tty")
    } else {
        Err(io::Error::new(io::ErrorKind::NotConnected, "not a tty"))
    }
}
