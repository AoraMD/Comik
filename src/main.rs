mod command;
mod context;
mod model;
mod util;

use clap::{AppSettings, Parser, Subcommand};
use command::execute;
use context::Context;
use std::path::PathBuf;
use util::mail;

pub const APP_NAME: &str = "comik";
pub const APP_NAME_TITALIZE: &str = "Comik";

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub const APP_NAME_IN_PATH: &str = APP_NAME_TITALIZE;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub const APP_NAME_IN_PATH: &str = APP_NAME;

#[cfg(target_os = "windows")]
fn default_cache_path() -> String {
    let path = match dirs::cache_dir() {
        Some(dir) => dir.join(APP_NAME_IN_PATH).join("cache"),
        None => {
            panic!("failed to load default cache dir");
        }
    };
    return path.to_string_lossy().to_string();
}

#[cfg(not(target_os = "windows"))]
fn default_cache_path() -> String {
    let path = match dirs::cache_dir() {
        Some(dir) => dir.join(APP_NAME_IN_PATH),
        None => {
            panic!("failed to load default cache dir");
        }
    };
    return path.to_string_lossy().to_string();
}

#[cfg(target_os = "windows")]
fn default_repo_path() -> String {
    let path = match dirs::data_local_dir() {
        Some(dir) => dir.clone().join(APP_NAME_IN_PATH).join("repo"),
        None => {
            panic!("failed to load default repo dir");
        }
    };
    return path.to_string_lossy().to_string();
}

#[cfg(not(target_os = "windows"))]
fn default_repo_path() -> String {
    let path = match dirs::data_local_dir() {
        Some(dir) => dir.clone().join(APP_NAME_IN_PATH),
        None => {
            panic!("failed to load default repo dir");
        }
    };
    return path.to_string_lossy().to_string();
}

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
        default_value_t = default_cache_path()
    )]
    cache: String,

    /// Set repository directory path
    #[clap(
        short,
        long,
        value_parser,
        value_name = "path",
        default_value_t = default_repo_path()
    )]
    repo: String,

    #[clap(subcommand)]
    command: Option<Command>,
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
    async fn execute(self, context: &Context) {
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
                    execute::main(learn, scale, config, context).await;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();
    let context = Context::new(
        args.debug,
        PathBuf::from(args.cache.as_str()),
        PathBuf::from(args.repo.as_str()),
        args.bark.clone(),
    );
    context.report_debug(&format!(
        "[args] cache directory path: {}",
        &args.cache
    ));
    context.report_debug(&format!(
        "[args] repository directory path: {}",
        &args.repo
    ));
    context.report_debug(&format!(
        "[args] Bark URL: {}",
        &args.bark.unwrap_or("null".to_string())
    ));
    args.command.unwrap().execute(&context).await;
}
