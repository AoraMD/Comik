mod command;
mod context;
mod model;
mod util;

use clap::{AppSettings, Parser, Subcommand};
use command::execute;
use context::Context;
use std::path::PathBuf;
use util::mail;

pub const APP_NAME: &str = "Comik";

#[derive(Parser)]
#[clap(author, version, about)]
#[clap(global_setting(AppSettings::ArgRequiredElseHelp))]
struct Args {
    /// Enable debug output
    #[clap(short, long, action)]
    debug: bool,

    /// Bark notification URL
    #[clap(short, long, value_name = "url")]
    bark: Option<String>,

    /// Set program cache directory path
    #[clap(
        short,
        long,
        value_parser,
        value_name = "path",
        default_value = "/var/cache/comik"
    )]
    cache: PathBuf,

    /// Set repository directory path
    #[clap(
        short,
        long,
        value_parser,
        value_name = "path",
        default_value = "/var/local/comik"
    )]
    repo: PathBuf,

    #[clap(subcommand)]
    subcommand: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Execute {
        /// Mark but skip downloading matches
        #[clap(short, long, action)]
        learn: bool,

        /// Set scale factor of comic image and document page size
        #[clap(short, long, value_name = "factor", default_value_t = 0.9)]
        scale: f64,

        /// Set config file path
        #[clap(short, long, value_parser, value_name = "file")]
        config: PathBuf,
    },
}

impl Command {
    fn execute(self, context: &Context) {
        match self {
            Command::Execute {
                learn,
                scale,
                config,
            } => {
                context.report_debug("run command: execute");
                context.report_debug(&format!("[args::execulte] learn: {}", learn));
                context.report_debug(&format!("[args::execulte] scale: {}", scale));
                context.report_debug(&format!(
                    "[args::execulte] config file path: {}",
                    config.display()
                ));
                if scale > 1.0 || scale < 0.0 {
                    context.report_error("scale factor must be between 0.0 and 1.0");
                } else {
                    execute::execute_main(learn, scale, config, context);
                }
            }
        }
    }
}

fn main() {
    let args: Args = Args::parse();
    let context = Context::new(
        args.debug,
        args.cache.clone(),
        args.repo.clone(),
        args.bark.clone(),
    );
    context.report_debug(&format!(
        "[args] cache directory path: {}",
        &args.cache.display()
    ));
    context.report_debug(&format!(
        "[args] repository directory path: {}",
        &args.cache.display()
    ));
    context.report_debug(&format!(
        "[args] Bark URL: {}",
        &args.bark.unwrap_or("null".to_string())
    ));
    args.subcommand.unwrap().execute(&context);
}
