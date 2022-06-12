use async_trait::async_trait;
use log::debug;

use wekan_common::artifact::common::{AType, Artifact, SortedArtifact};

use crate::error::kind::Error;

#[async_trait]
pub trait Unwrapper {
    async fn get_result(res: Result<Vec<Artifact>, Error>) -> Vec<Artifact> {
        match res {
            Ok(res) => res,
            Err(_e) => Vec::<Artifact>::new(),
        }
    }
}

pub trait SatisfyType {
    fn satisfy(&mut self, atype: AType);
}
impl<A: SortedArtifact + std::fmt::Debug> SatisfyType for Vec<A> {
    fn satisfy(&mut self, atype: AType) {
        let iter = self.iter_mut();
        for el in iter {
            el.set_type(atype.clone());
        }
        debug!("Response: {:?}", self);
    }
}
