use crate::{command::WekanParser, error::kind::Error, result::kind::WekanResult};
use clap::Parser;
use log::{debug, error, trace};
use wekan_common::artifact::common::{Base, BaseDetails, MostDetails, SortedArtifact};
use wekan_core::error::kind::Error as CoreError;

pub trait CliDisplay {
    fn print_debug(artifact_details: String) -> Result<WekanResult, Error> {
        WekanResult::new_msg(&artifact_details).ok()
    }
    fn print_most_details<
        T: SortedArtifact + std::fmt::Debug + BaseDetails + Base + MostDetails,
    >(
        artifact_details: T,
    ) -> Result<WekanResult, Error> {
        trace!("{:?}", artifact_details);

        let mut id_to_print = String::new();
        let mut headline = String::from("ID");
        let det = artifact_details.get_id();
        let (tmp, _last) = det.split_at(4);
        id_to_print.push_str(tmp);
        headline.push_str(&" ".repeat(5));
        headline.push_str("TITLE");
        headline.push_str(&" ".repeat(3));
        if artifact_details.get_title().len() >= 3 {
            headline.push_str(&" ".repeat(artifact_details.get_title().len() - 3));
        } else {
            headline.push_str(&" ".repeat(artifact_details.get_title().len()));
        }
        headline.push_str("DESCRIPTION");
        headline.push_str(&" ".repeat(3));
        headline.push_str("CREATED AT");
        headline.push_str(&" ".repeat(3));
        headline.push_str("DUE AT");
        headline.push_str(&" ".repeat(7));
        headline.push_str("END AT");
        println!("{}", headline);
        #[cfg(feature = "integration")]
        println!("AAAA   {}", artifact_details.get_title());
        let created_at_long = artifact_details.get_created_at();
        let (created_at, _last) = created_at_long.split_at(10);
        let due_at_long = artifact_details.get_due_at();
        let (due_at, _last) = due_at_long.split_at(10);
        let end_at_long = artifact_details.get_end_at();
        let (end_at, _last) = end_at_long.split_at(10);
        #[cfg(not(feature = "integration"))]
        println!(
            "{}   {}{}   {}      {}   {}   {}",
            id_to_print,
            artifact_details.get_title(),
            " ".repeat(artifact_details.get_title().len() + 3),
            artifact_details.get_description(),
            created_at,
            due_at,
            end_at
        );
        let mut details_type = artifact_details.get_type().to_string();
        details_type.push_str(" details completed.");
        WekanResult::new_workflow(
            &details_type.to_string(),
            "Update the specified artifact with the subcommand 'update'",
        )
        .ok()
    }

    fn print_details<T: SortedArtifact + std::fmt::Debug + BaseDetails + Base>(
        artifact_details: T,
        format: Option<String>,
    ) -> Result<WekanResult, Error> {
        debug!("print_details with {:?}", format);
        trace!("{:?}", artifact_details);

        let mut id_to_print = String::new();
        let mut headline = String::from("ID");

        headline.push_str(&" ".repeat(2));
        match format {
            Some(f) => {
                debug!("{:?}", f);
                if f.starts_with("long") {
                    debug!("Format is long.");
                    headline.push_str(&" ".repeat(artifact_details.get_id().len()));
                    id_to_print.push_str(&artifact_details.get_id());
                } else {
                    debug!("Format is sth else.");
                    headline.push_str(&" ".repeat(3));
                    let det = artifact_details.get_id();
                    let (tmp, _last) = det.split_at(4);
                    id_to_print.push_str(tmp);
                }
            }
            None => {
                debug!("Format is not set.");
                headline.push_str(&" ".repeat(3));
                let det = artifact_details.get_id();
                let (tmp, _last) = det.split_at(4);
                id_to_print.push_str(tmp);
            }
        };

        headline.push_str("TITLE");
        headline.push_str(&" ".repeat(3));
        headline.push_str("MODIFIED AT");
        headline.push_str(&" ".repeat(3));
        let modified_at_long = artifact_details.get_modified_at();
        let (modified_at, _last) = modified_at_long.split_at(10);
        let created_at_long = artifact_details.get_created_at();
        let (created_at, _last) = created_at_long.split_at(10);
        headline.push_str("CREATED AT");
        println!("{}", headline);
        #[cfg(feature = "integration")]
        println!("AAAA   {}", artifact_details.get_title());
        #[cfg(not(feature = "integration"))]
        println!(
            "{}   {}{}   {}   {}",
            id_to_print,
            artifact_details.get_title(),
            " ".repeat(2),
            modified_at,
            created_at
        );
        let mut details_type = artifact_details.get_type().to_string();
        details_type.push_str(" details completed.");
        WekanResult::new_workflow(
            &details_type.to_string(),
            "Update the specified artifact with the subcommand 'update'",
        )
        .ok()
    }
    fn print_artifacts<T: std::fmt::Debug + Base + std::fmt::Display>(
        artifacts: Vec<T>,
        format: String,
    ) -> Result<WekanResult, Error> {
        let mut iterator = artifacts.iter();

        trace!("{:?} - {:?}", artifacts, format);
        debug!("print_artifacts");
        if !artifacts.is_empty() {
            println!("ID     TITLE");
            loop {
                match iterator.next() {
                    Some(r) => {
                        if format.contains("rust") {
                            println!("{:?}", r);
                        } else if format.contains("elisp") {
                            println!("{}", r);
                        } else {
                            let tmp = r.get_id().to_owned();
                            let (_first, _last) = tmp.split_at(4);
                            #[cfg(feature = "integration")]
                            println!("AAAA   {}", r.get_title());
                            #[cfg(not(feature = "integration"))]
                            println!("{}   {}", _first, r.get_title());
                        };
                    }
                    None => {
                        if format.contains("elisp") {
                            break WekanResult::new_msg(&String::new()).ok();
                        } else {
                            break WekanResult::new_workflow(
                                "Artifact printed",
                                "Get or update details of an artifact.",
                            )
                            .ok();
                        }
                    }
                }
            }
        } else {
            WekanResult::new_workflow(
                "This artifact contains no childs.",
                "Create a card with 'card -b [BOARD-NAME] -l [LIST-NAME] create [CARD-NAME] --description [CARD-DESCRIPTION]'").ok()
        }
    }

    fn print_table<
        T: std::fmt::Debug
            + std::cmp::PartialOrd
            + std::cmp::Ord
            + SortedArtifact
            + Base
            + std::fmt::Display,
    >(
        lists: Vec<T>,
        mut cards: Vec<Vec<T>>,
    ) -> Result<WekanResult, Error> {
        let mut iterator = lists.iter();
        if !lists.is_empty() {
            loop {
                match iterator.next() {
                    Some(r) => print!("{}        ", r.get_title()),
                    None => break println!(),
                }
            }
        };
        if !cards.is_empty() {
            let mut iterator = cards.iter_mut();
            loop {
                match iterator.next() {
                    Some(r) => {
                        r.sort_by(|a, b| a.cmp(b));
                        trace!("{:?}", r);
                        let mut inner_cards = r.iter();
                        loop {
                            match inner_cards.next() {
                                Some(a) => {
                                    println!("{}        ", a.get_title())
                                }
                                None => break println!(),
                            }
                        }
                    }
                    None => break println!(),
                }
            }
        };
        WekanResult::new_msg("Table printed").ok()
    }
    fn transform_to_exit(result: Result<WekanResult, Error>) -> i8 {
        debug!("transform_to_exit");
        trace!("{:?}", result);
        match result {
            Ok(r) => {
                println!("{}", r.message);
                let parser = WekanParser::parse();
                if !parser.delegate.no_recommendations {
                    println!("The next recommended workflows:");
                    match &r.next_workflow {
                        Some(w) => println!("{}", w),
                        None => println!("Nothing to recommend. Suggestions?"),
                    };
                }
                r.exit_code
            }
            Err(e) => {
                eprint!("Something went wrong. ");
                eprint!(
                    "For more information use WEKAN_LOG, verbose argument or WEKAN_BACKTRACE=1. "
                );
                eprintln!("Trying to make sense of the error and showing user friendly output:");
                trace!("{:?}", e);
                match e {
                    Error::Core(core) => match core {
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
                                "Config file or Context file not found. Check your WEKAN_PATH."
                            );
                            2
                        }
                        CoreError::Yaml(yaml) => {
                            error!("{:?}", yaml);
                            eprintln!(
                                "Config or Context was not loaded successfully. Delete WEKAN_PATH."
                            );
                            2
                        }
                    },
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
}
