use std::{
    io::{self, Read, Write},
    time::{Duration, Instant},
};

use kursor_core::term_info::TermInfo;
use windows_sys::Win32::{
    Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::{
        Console::{
            ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT,
            ENABLE_VIRTUAL_TERMINAL_INPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
            GetConsoleMode, GetStdHandle, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
            SetConsoleMode,
        },
        Threading::WaitForSingleObject,
    },
};

use super::{ProbeParser, ProbeSession, direct_queries};

pub fn query(timeout: Duration, info: &mut TermInfo) -> io::Result<bool> {
    let mut input = io::stdin();
    let mut output = io::stdout();
    let input_handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    let output_handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    if input_handle.is_null() || output_handle.is_null() {
        return Ok(false);
    }

    let mut input_mode = 0;
    let mut output_mode = 0;
    if unsafe { GetConsoleMode(input_handle, &mut input_mode) } == 0
        || unsafe { GetConsoleMode(output_handle, &mut output_mode) } == 0
    {
        return Ok(false);
    }

    let input_mode_new = (input_mode
        & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT))
        | ENABLE_VIRTUAL_TERMINAL_INPUT;
    let output_mode_new = output_mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING;

    if unsafe { SetConsoleMode(input_handle, input_mode_new) } == 0 {
        return Ok(false);
    }
    if unsafe { SetConsoleMode(output_handle, output_mode_new) } == 0 {
        let _ = unsafe { SetConsoleMode(input_handle, input_mode) };
        return Ok(false);
    }

    let result = exchange(&mut input, &mut output, input_handle, timeout, info);

    let input_restore = unsafe { SetConsoleMode(input_handle, input_mode) };
    let output_restore = unsafe { SetConsoleMode(output_handle, output_mode) };
    let route = result?;
    if input_restore == 0 || output_restore == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(route)
}

fn exchange(
    input: &mut impl Read,
    output: &mut impl Write,
    input_handle: windows_sys::Win32::Foundation::HANDLE,
    timeout: Duration,
    info: &mut TermInfo,
) -> io::Result<bool> {
    let deadline = Instant::now() + timeout;
    let mut parser = ProbeParser::default();
    let mut session = ProbeSession::new(info);
    let mut buffer = [0u8; 4096];

    output.write_all(direct_queries(session.info(), false).as_bytes())?;
    output.flush()?;
    let _ = read(
        input,
        input_handle,
        deadline,
        &mut session,
        &mut parser,
        &mut buffer,
    )?;

    Ok(false)
}

fn read(
    input: &mut impl Read,
    input_handle: windows_sys::Win32::Foundation::HANDLE,
    deadline: Instant,
    session: &mut ProbeSession<'_>,
    parser: &mut ProbeParser,
    buffer: &mut [u8; 4096],
) -> io::Result<bool> {
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(false);
        }
        let milliseconds = remaining.as_millis().min(u32::MAX as u128) as u32;
        let status = unsafe { WaitForSingleObject(input_handle, milliseconds) };
        if status == WAIT_TIMEOUT {
            return Ok(false);
        }
        if status != WAIT_OBJECT_0 {
            return Err(io::Error::last_os_error());
        }

        let count = input.read(buffer)?;
        if count == 0 {
            return Ok(false);
        }
        if session.accept(parser.feed(&buffer[..count])) {
            return Ok(true);
        }
    }
}
