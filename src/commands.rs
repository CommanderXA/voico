use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "Voijix")]
#[clap(about = "A voice journal and audio analysis toolkit", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Clones repos
    Record {
        /// A name of clip to be recorded
        name: Option<String>,
    },
    /// Lists all clips
    List {},
    /// Plays a recording
    #[clap(arg_required_else_help = true)]
    Play {
        /// A name of the clip to be played
        name: String,
    },
    #[clap(arg_required_else_help = true)]
    Delete {
        /// Deletes a clip
        name: String,
    },
}
