extern crate env_logger;
extern crate failure;
extern crate mdbook;
extern crate mdbook_linkcheck;
extern crate pulldown_cmark;
extern crate serde_json;
#[macro_use]
extern crate structopt;

use failure::{Error, ResultExt, SyncFailure};
use mdbook::renderer::RenderContext;
use mdbook::MDBook;
use mdbook_linkcheck::BrokenLinks;
use std::env;
use std::io;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

fn main() {
    env_logger::init();
    let args = Args::from_args();

    if let Err(e) = run(&args) {
        if let Some(broken_links) = e.downcast_ref::<BrokenLinks>() {
            if broken_links.0.len() == 1 {
                eprintln!("There was 1 broken link:");
            } else {
                eprintln!("There were {} broken links:", broken_links.0.len());
            }

            for error in &broken_links.0 {
                eprintln!("{}#{}: {}", error.chapter().display(), error.line(), error);
            }
        } else {
            eprintln!("Error: {}", e);

            for cause in e.iter_chain() {
                eprintln!("\tCaused By: {}", cause);
            }

            if env::var("RUST_BACKTRACE").is_ok() {
                eprintln!();
                eprintln!("{}", e.backtrace());
            }
        }

        process::exit(1);
    }
}

fn run(args: &Args) -> Result<(), Error> {
    // get a `RenderContext`, either from stdin (because we're used as a plugin)
    // or by instrumenting MDBook directly (in standalone mode).
    let ctx: RenderContext = if args.standalone {
        let md = MDBook::load(&args.root).map_err(SyncFailure::new)?;
        let destination = md.build_dir_for("epub");

        RenderContext {
            // TODO: Pull this from mdbook instead of hard-coding
            version: String::from("0.1.3"),
            root: md.root,
            book: md.book,
            config: md.config,
            destination: destination,
        }
    } else {
        serde_json::from_reader(io::stdin()).context("Unable to parse RenderContext")?
    };

    mdbook_linkcheck::check_links(&ctx)?;

    Ok(())
}

#[derive(Debug, Clone, StructOpt)]
struct Args {
    #[structopt(
        short = "s",
        long = "standalone",
        help = "Run standalone (i.e. not as a mdbook plugin)"
    )]
    standalone: bool,
    #[structopt(
        help = "The book to render.",
        parse(from_os_str),
        default_value = "."
    )]
    root: PathBuf,
}
