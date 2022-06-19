use clap::Parser;
use log::{debug, error, info, trace};
use wekan_cli::{
    command::WekanParser, error::kind::Error, result::kind::WekanResult, runner::Runner,
};
use wekan_core::error::kind::Error as CoreError;
#[tokio::main]
async fn main() {
    let parser = WekanParser::parse();
    let mut c = Runner::new(parser.delegate, parser.command).await;
    std::process::exit(<Runner as ExitCode>::transform_to_exit(c.run().await).into());
}

trait ExitCode {
    fn transform_to_exit(result: Result<WekanResult, Error>) -> i8 {
        debug!("transform_to_exit");
        trace!("{:?}", result);
        match result {
            Ok(r) => {
                println!("{}", r.get_msg().trim());
                let parser = WekanParser::parse();
                if !parser.delegate.no_recommendations {
                    match &r.get_next_workflow() {
                        Some(w) => println!("Recommended workflow: {}", w.trim()),
                        None => {}
                    };
                }
                r.get_exit_code()
            }
            Err(e) => {
                debug!("{:?}", e);
                info!("Use WEKAN_LOG, verbose or WEKAN_BACKTRACE=1.");
                match e {
                    Error::Core(core) => Self::transform_core_error(core),
                    Error::Cli(cli) => {
                        eprintln!("{}", cli.message);
                        cli.error_code
                    }
                    Error::Input(i) => {
                        println!("{}", i.message);
                        0
                    }
                    Error::Io(io) => {
                        eprintln!("IO Error");
                        eprint!("{:?}", io);
                        3
                    }
                    Error::Yaml(yaml) => {
                        eprintln!("{:?}", yaml);
                        4
                    }
                    #[cfg(feature = "store")]
                    Error::Store(store) => {
                        eprintln!("{:?}", store);
                        4
                    }
                }
            }
        }
    }

    fn transform_core_error(err: CoreError) -> i8 {
        match err {
            CoreError::Constraint(c) => {
                eprintln!("{:?}", c);
                1
            }
            CoreError::Http(h) => {
                if h.is_timeout() || h.is_connect() {
                    eprintln!("Probably host down or Port not open.");
                }
                if h.is_request() {
                    eprintln!("Either payload failure or authentication issue.");
                }
                if h.is_body() {
                    eprintln!("Wrong payload or response. WEKAN_CLI works best against API v6.11.");
                }

                if h.is_decode() {
                    eprintln!("Response couldn't be decoded. Check WEKAN_API version.");
                }

                2
            }

            CoreError::Io(io) => {
                error!("{:?}", io);
                eprintln!(
                    "Config file or Context file not found. Check your WEKAN_CLI_CONFIG_PATH."
                );
                2
            }
            CoreError::Yaml(yaml) => {
                error!("{:?}", yaml);
                eprintln!("Failed to load config or context. Delete WEKAN_CLI_CONFIG_PATH.");
                2
            }
        }
    }
}
impl ExitCode for Runner {}
