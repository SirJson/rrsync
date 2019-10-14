extern crate clap;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate rssync;

use clap::{App, Arg, SubCommand};
use std::env;
use std::path::Path;

use rssync::{Error, Index, IndexTransaction};
use rssync::locations::Location;

/// Command-line entrypoint
fn main() {
    // Parse command line
    let cli = App::new("rssync")
        .bin_name("rssync")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .help("Augment verbosity (print more details)")
                .multiple(true),
        )
        .subcommand(
            SubCommand::with_name("index")
                .about("Index a file or directory")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("index-file")
                        .short("x")
                        .takes_value(true)
                        .default_value("rssync.idx"),
                ),
        )
        .subcommand(
            SubCommand::with_name("sync")
                .about("Copy files")
                .arg(
                    Arg::with_name("source")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("destination")
                        .required(true)
                        .takes_value(true),
                )
        );

    let mut cli = cli;
    let matches = match cli.get_matches_from_safe_borrow(env::args_os()) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(2);
        }
    };

    // Set up logging
    {
        let level = match matches.occurrences_of("verbose") {
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        };
        let mut logger_builder = env_logger::builder();
        logger_builder.filter(None, level);
        if let Ok(val) = env::var("RSSYNC_LOG") {
            logger_builder.parse_filters(&val);
        }
        if let Ok(val) = env::var("RSSYNC_LOG_STYLE") {
            logger_builder.parse_write_style(&val);
        }
        logger_builder.init();
    }

    let res = match matches.subcommand_name() {
        Some("index") => || -> Result<(), Error> {
            let s_matches = matches.subcommand_matches("index").unwrap();
            let index_filename = s_matches.value_of_os("index-file").unwrap();
            let index_filename = Path::new(index_filename);
            let path = Path::new(s_matches.value_of_os("path").unwrap());

            let mut index = Index::open(index_filename.into())?;
            let mut index_tx = index.transaction()?;
            index_path(&mut index_tx, path)?;
            remove_missing_files(&mut index_tx, path)?;
            index_tx.commit()?;

            Ok(())
        }(),
        Some("sync") => {
            let s_matches = matches.subcommand_matches("index").unwrap();
            let source = s_matches.value_of_os("source").unwrap();
            let dest = s_matches.value_of_os("destination").unwrap();

            let source = match source.to_str().and_then(Location::parse) {
                Some(s) => s,
                None => {
                    eprintln!("Invalid source");
                    std::process::exit(2);
                }
            };
            let dest = match dest.to_str().and_then(Location::parse) {
                Some(s) => s,
                None => {
                    eprintln!("Invalid destination");
                    std::process::exit(2);
                }
            };

            match (source, dest) {
                (Location::Local(_), Location::Local(_)) => unimplemented!(),
                (Location::Local(_), Location::Ssh { .. }) => unimplemented!(),
                (Location::Ssh { .. }, Location::Local(_)) => unimplemented!(),
                (Location::Http(_), Location::Local(_)) => unimplemented!(),
                // Unsupported variants
                (Location::Http(_), _) => {
                    eprintln!(
                        "HTTP download is only supported to local files"
                    );
                    std::process::exit(2);
                }
                (_, Location::Http(_)) => {
                    eprintln!("Cannot upload to HTTP");
                    std::process::exit(2);
                }
                (Location::Ssh { .. }, Location::Ssh { .. }) => {
                    // FIXME: Could we support this?
                    eprintln!(
                        "Direct transfer between remote hosts is not supported"
                    );
                    std::process::exit(2);
                }
            }
        }
        _ => {
            cli.print_help().expect("Can't print help");
            std::process::exit(2);
        }
    };

    if let Err(e) = res {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

/// Recursively descends in directories and add all files to the index
fn index_path<'a>(
    index: &mut IndexTransaction<'a>,
    path: &Path,
) -> Result<(), Error>
{
    if path.is_dir() {
        info!("Indexing directory {:?}", path);
        for entry in path.read_dir()? {
            if let Ok(entry) = entry {
                index_path(index, &entry.path())?;
            }
        }
        Ok(())
    } else {
        let path = if path.starts_with(".") {
            path.strip_prefix(".").unwrap()
        } else {
            path
        };
        info!("Indexing file {:?}", path);
        index.index_file(&path)
    }
}

/// List all files and remove those that don't exist on disk
fn remove_missing_files<'a>(
    index: &mut IndexTransaction<'a>,
    path: &Path,
) -> Result<(), Error>
{
    for (file_id, file_path) in index.list_files()? {
        if !path.join(&file_path).is_file() {
            info!("Removing missing file {:?}", file_path);
            index.remove_file(file_id)?;
        }
    }
    Ok(())
}
