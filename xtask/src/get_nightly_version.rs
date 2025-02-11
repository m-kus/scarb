use anyhow::Result;
use cairo_toolchain_xtasks::sync_version::expected_version;
use clap::Parser;
use semver::{BuildMetadata, Prerelease, Version};
use time::OffsetDateTime;

#[derive(Parser)]
pub struct Args {
    #[arg(short, long)]
    tag: bool,
}

pub fn main(args: Args) -> Result<()> {
    let tag = nightly_tag();
    if args.tag {
        println!("{tag}");
    } else {
        let version = nightly_version()?;
        println!("{version}");
    }
    Ok(())
}

pub fn nightly_version() -> Result<Version> {
    let mut version = expected_version()?;
    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::new(&nightly_tag()).unwrap();
    Ok(version)
}

pub fn nightly_tag() -> String {
    let dt = OffsetDateTime::now_utc();
    format!(
        "nightly-{}-{:0>2}-{:0>2}",
        dt.year(),
        u8::from(dt.month()),
        dt.day()
    )
}
