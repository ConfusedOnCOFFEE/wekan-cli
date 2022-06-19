use crate::{error::kind::Error, result::kind::WekanResult};
use log::{debug, trace};
use std::cmp::Ordering;
use wekan_common::artifact::common::{
    Base, BaseDetails, IdReturner, MostDetails, SortedArtifact, WekanDisplay,
};

#[cfg(test)]
use std::io::Write;

pub struct CliDisplay {
    writer: Vec<u8>,
}

impl std::clone::Clone for CliDisplay {
    fn clone(&self) -> Self {
        Self {
            writer: self.writer.clone(),
        }
    }
}
impl CliDisplay {
    pub fn new(writer: Vec<u8>) -> Self {
        Self { writer }
    }

    fn format(&mut self, msg: &str, width: usize) -> String {
        let padded = format!("{:<width$}", msg, width = width + 3);
        #[cfg(test)]
        self.capture_out(&padded);
        padded
    }

    #[cfg(test)]
    fn capture_out(&mut self, msg: &str) {
        self.writer.write(msg.as_bytes()).unwrap();
    }

    #[cfg(test)]
    fn get_captured(mut self) -> String {
        let res = String::from_utf8(self.writer.to_owned())
            .unwrap()
            .to_owned();
        self.writer.flush().unwrap();
        res
    }

    pub fn print_most_details<T: WekanDisplay + BaseDetails + MostDetails>(
        &mut self,
        artifact_details: T,
    ) -> Result<WekanResult, Error> {
        trace!("{:?}", artifact_details);
        let mut properties_to_show = vec![
            artifact_details.get_id(),
            artifact_details.get_title(),
            artifact_details
                .get_modified_at()
                .split_once('T')
                .unwrap()
                .0
                .to_string(),
            artifact_details
                .get_created_at()
                .split_once('T')
                .unwrap()
                .0
                .to_string(),
        ];
        properties_to_show.push(safely_unwrap_date(&artifact_details.get_end_at()));
        properties_to_show.push(safely_unwrap_date(&artifact_details.get_due_at()));
        let properties_iter = properties_to_show.iter();
        let max_string = properties_iter.max_by(|x, y| cmp_by_length(x, y)).unwrap();
        let mut output = String::new();
        let mut headlines_to_show = vec![
            String::from("ID"),
            String::from("TITLE"),
            String::from("MODIFIED_AT"),
            String::from("CREATED_AT"),
        ];
        headlines_to_show.push(if_field_available(
            &String::from("DUE_AT"),
            &artifact_details.get_due_at(),
        ));
        headlines_to_show.push(if_field_available(
            &String::from("END_AT"),
            &artifact_details.get_end_at(),
        ));
        headlines_to_show
            .iter()
            .for_each(|x| output.push_str(&self.format(x, max_string.len())));
        output = output.trim().to_string();
        output.push('\n');
        #[cfg(feature = "integration")]
        output.push_str(&format!("AAAA   {}", artifact_details.get_title()));
        #[cfg(not(feature = "integration"))]
        properties_to_show
            .iter()
            .for_each(|x| output.push_str(&self.format(x, max_string.len())));
        output = output.trim().to_string();
        WekanResult::new_workflow(
            &output.finish_up(),
            "Update the specified artifact with the subcommand 'update'",
        )
        .ok()
    }

    pub fn print_details<T: WekanDisplay + BaseDetails>(
        &mut self,
        artifact_details: T,
        format: Option<String>,
    ) -> Result<WekanResult, Error> {
        debug!("print_details_v2");
        let properties_to_show = vec![
            artifact_details
                .get_id()
                .split_at({
                    match format {
                        Some(f) => {
                            if f.starts_with("long")
                                || f.starts_with("extended")
                                || f.starts_with("extd")
                            {
                                artifact_details.get_id().len()
                            } else {
                                std::cmp::min(4, artifact_details.get_id().len())
                            }
                        }
                        None => std::cmp::min(4, artifact_details.get_id().len()),
                    }
                })
                .0
                .to_string(),
            artifact_details.get_title(),
            artifact_details
                .get_modified_at()
                .split_once('T')
                .unwrap()
                .0
                .to_string(),
            artifact_details
                .get_created_at()
                .split_once('T')
                .unwrap()
                .0
                .to_string(),
        ];
        let properties_iter = properties_to_show.iter();
        let max_string = properties_iter.max_by(|x, y| cmp_by_length(x, y)).unwrap();
        let headlines_to_show = vec![
            String::from("ID"),
            String::from("TITLE"),
            String::from("MODIFIED_AT"),
            String::from("CREATED_AT"),
        ];
        let mut output = String::new();
        headlines_to_show
            .iter()
            .for_each(|x| output.push_str(&self.format(x, max_string.len())));
        output = output.trim().to_string();
        output.push('\n');
        #[cfg(feature = "integration")]
        output.push_str(&format!("AAAA   {}", artifact_details.get_title()));
        #[cfg(not(feature = "integration"))]
        properties_to_show
            .iter()
            .for_each(|x| output.push_str(&self.format(x, max_string.len())));
        output = output.trim().to_string();
        WekanResult::new_msg(&output.finish_up()).ok()
    }
    pub fn print_artifacts<T: IdReturner + std::fmt::Debug + Base + std::fmt::Display>(
        &mut self,
        artifacts: Vec<T>,
        format: String,
    ) -> Result<WekanResult, Error> {
        trace!("{:?} - {:?}", artifacts, format);
        debug!("print_artifacts");
        let headlines_to_show = vec![String::from("ID"), String::from("TITLE")];
        let mut output = String::new();
        headlines_to_show
            .iter()
            .for_each(|x| output.push_str(&self.format(x, 3)));
        output = output.trim().to_string();
        output.push('\n');
        artifacts.iter().for_each(|a| {
            if format.contains("rust") || format.contains("elisp") {
                output.push_str(
                    &self.format(
                        a.get_id()
                            .split_at({
                                if format.starts_with("long")
                                    || format.starts_with("extended")
                                    || format.starts_with("extd")
                                {
                                    a.get_id().len()
                                } else {
                                    std::cmp::min(3, a.get_id().len())
                                }
                            })
                            .0,
                        3,
                    ),
                );
                output.push_str(&self.format(&a.get_title(), 3));
            } else if format.contains("extended") {
                output.push_str(&self.format(&a.get_id(), 3));
                output.push_str(&self.format(&a.get_title(), 3));
            } else {
                #[cfg(not(feature = "integration"))]
                output.push_str(
                    &self.format(
                        a.get_id()
                            .split_at({
                                if format.starts_with("long")
                                    || format.starts_with("extended")
                                    || format.starts_with("extd")
                                {
                                    a.get_id().len()
                                } else {
                                    std::cmp::min(4, a.get_id().len())
                                }
                            })
                            .0,
                        3,
                    ),
                );
                #[cfg(not(feature = "integration"))]
                output.push_str(&self.format(&a.get_title(), 3));
                #[cfg(feature = "integration")]
                output.push_str(&self.format(&String::from("AAAA"), 3));
                #[cfg(feature = "integration")]
                output.push_str(&self.format(&a.get_title(), 3));
                output = output.trim().to_string();
                output.push('\n');
            };
        });
        WekanResult::new_workflow(&output.finish_up(), "Get or update details of an artifact.").ok()
    }

    pub fn prepare_output<T: IdReturner + std::fmt::Debug + Base + std::fmt::Display>(
        &mut self,
        output: &str,
        artifacts: Vec<T>,
        format: String,
    ) -> Result<WekanResult, Error> {
        let mut full_output = String::new();
        full_output.push_str(output);
        let second_output = self.print_artifacts(artifacts, format).unwrap();
        full_output.push_str(&second_output.get_msg());
        WekanResult::new_workflow(&full_output, &second_output.get_next_workflow().unwrap()).ok()
    }
    pub fn print_table<
        T: std::fmt::Debug
            + std::cmp::PartialOrd
            + std::cmp::Ord
            + SortedArtifact
            + Base
            + std::fmt::Display,
    >(
        &mut self,
        lists: Vec<T>,
        mut cards: Vec<Vec<T>>,
    ) -> Result<WekanResult, Error> {
        let mut output = String::new();
        lists
            .iter()
            .for_each(|x| output.push_str(&self.format(&x.get_title(), 3)));
        output.push('\n');
        if !cards.is_empty() {
            let mut iterator = cards.iter_mut();
            loop {
                match iterator.next() {
                    Some(r) => {
                        trace!("{:?}", r);
                        if !r.is_empty() {
                            let next_card = r.remove(0);
                            output.push_str(&self.format(&next_card.get_title(), 3))
                        }
                    }
                    None => break,
                }
            }
        };
        WekanResult::new_msg(&output.finish_up()).ok()
    }
}
fn cmp_by_length(x: &str, y: &str) -> Ordering {
    if x.len() > y.len() {
        return Ordering::Greater;
    };
    if x.len() == y.len() {
        Ordering::Equal
    } else {
        Ordering::Less
    }
}

fn safely_unwrap_date(d: &str) -> String {
    match d.split_once('T') {
        Some(e) => e.0.to_string(),
        None => String::new(),
    }
}

fn if_field_available(h: &str, field: &str) -> String {
    if !field.is_empty() {
        h.to_string()
    } else {
        String::new()
    }
}

trait FinishUp {
    fn finish_up(&mut self) -> String;
}

impl FinishUp for String {
    fn finish_up(&mut self) -> String {
        self.push_str("\n----\n");
        self.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wekan_common::artifact::{card::Details as CDetails, tests::MockDetails};

    #[test]
    fn print_details_output_normal() {
        let a = CDetails::mock("my-id", "my-title", "2022-10-15T208Z");
        let out = Vec::new();
        let mut display = CliDisplay::new(out);
        let ok_res = display.print_details(a, None).unwrap();
        let expected_output = concat!(
            "ID           TITLE        MODIFIED_AT  CREATED_AT   ",
            "my-i         my-title     2022-10-15   2022-10-15   ",
        );
        let expected_msg = concat!(
            "ID           TITLE        MODIFIED_AT  CREATED_AT\n",
            "my-i         my-title     2022-10-15   2022-10-15\n----\n",
        );
        assert_eq!(
            display.get_captured().escape_debug().to_string(),
            expected_output
        );
        assert_eq!(ok_res.get_msg(), expected_msg);
        assert_eq!(ok_res.get_next_workflow(), None);
        assert_eq!(ok_res.get_exit_code(), 0)
    }

    #[test]
    fn cmp_by_length_greater() {
        assert_eq!(cmp_by_length("202", "2"), Ordering::Greater)
    }

    #[test]
    fn cmp_by_length_less() {
        assert_eq!(cmp_by_length("2", "202"), Ordering::Less)
    }

    #[test]
    fn cmp_by_length_equal() {
        assert_eq!(cmp_by_length("20", "20"), Ordering::Equal)
    }

    #[test]
    fn safely_unwrap_date_exist() {
        assert_eq!(safely_unwrap_date("202T27Z"), String::from("202"))
    }

    #[test]
    fn safely_unwrap_date_dont_exist() {
        assert_eq!(safely_unwrap_date("20"), "")
    }

    #[test]
    fn if_field_vailable_true() {
        assert_eq!(
            if_field_available("HEADER", "header"),
            String::from("HEADER")
        )
    }
    #[test]
    fn if_field_vailable_false() {
        assert_eq!(if_field_available("HEADER", ""), String::new())
    }
}
