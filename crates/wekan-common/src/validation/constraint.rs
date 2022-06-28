use crate::artifact::common::Artifact;

use super::user::User;

#[derive(Debug, Clone)]
pub enum Constraint {
    Board(BoardConstraint),
    List(ListConstraint),
    Card(CardConstraint),
    Checklist(ChecklistConstraint),
    Login(bool),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BoardConstraint {
    pub user: Result<User, bool>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CardConstraint {
    pub board: Artifact,
    pub list: Artifact,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ListConstraint {
    pub board: Artifact,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SwimlaneConstraint {
    pub board: Artifact,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ChecklistConstraint {
    pub board: Artifact,
    pub list: Artifact,
    pub card: Artifact,
}
