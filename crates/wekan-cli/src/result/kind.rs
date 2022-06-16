use crate::error::kind::Error;
#[derive(Debug)]
pub struct WekanResult {
    message: String,
    exit_code: i8,
    next_workflow: Option<String>,
}

impl WekanResult {
    pub fn new_msg(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
            exit_code: 0,
            next_workflow: None,
        }
    }

    pub fn new_workflow(msg: &str, workflow: &str) -> Self {
        Self {
            message: msg.to_string(),
            exit_code: 0,
            next_workflow: Some(workflow.to_string()),
        }
    }

    pub fn new_exit(msg: &str, exit_code: i8, next_workflow: Option<String>) -> Self {
        Self {
            message: msg.to_string(),
            exit_code,
            next_workflow,
        }
    }
    pub fn ok(&self) -> Result<WekanResult, Error> {
        Ok(self.clone())
    }

    pub fn get_msg(&self) -> String {
        self.message.to_owned()
    }

    pub fn get_next_workflow(&self) -> Option<String> {
        self.next_workflow.to_owned()
    }

    pub fn get_exit_code(&self) -> i8 {
        self.exit_code
    }
}

impl Clone for WekanResult {
    fn clone(&self) -> Self {
        Self {
            message: self.message.to_owned(),
            exit_code: self.exit_code,
            next_workflow: self.next_workflow.clone(),
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.message = source.message.to_owned();
        self.exit_code = source.exit_code;
        self.next_workflow = source.next_workflow.clone()
    }
}
