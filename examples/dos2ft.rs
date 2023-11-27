// SPDX-FileCopyrightText: 2023 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! An example of converting DOS date and time to the file time.

// Lint levels of rustc.
#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]
#![warn(rust_2018_idioms)]
// Lint levels of Clippy.
#![warn(clippy::cargo, clippy::nursery, clippy::pedantic)]

#[cfg(feature = "std")]
use anyhow::Context;
#[cfg(feature = "std")]
use clap::Parser;

#[cfg(feature = "std")]
#[derive(Debug, Parser)]
#[command(version, about)]
struct Opt {
    /// Additional finer resolution of DOS date and time.
    #[arg(short, long)]
    resolution: Option<u8>,

    /// UTC offset of DOS date and time.
    #[arg(short, long, allow_hyphen_values(true))]
    offset: Option<i8>,

    /// DOS date to convert.
    date: u16,

    /// DOS time to convert.
    time: u16,
}

#[cfg(feature = "std")]
fn main() -> anyhow::Result<()> {
    use nt_time::{time::UtcOffset, FileTime};

    let opt = Opt::parse();

    let offset = opt
        .offset
        .map(|o| UtcOffset::from_whole_seconds(i32::from(o) * 900))
        .transpose()
        .context("could not create the UTC offset")?;
    let ft = FileTime::from_dos_date_time(opt.date, opt.time, opt.resolution, offset)
        .context("could not convert date and time")?;
    println!("{ft}");
    Ok(())
}

#[cfg(not(feature = "std"))]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("`std` feature is required");
}
