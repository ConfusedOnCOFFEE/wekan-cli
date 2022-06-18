use log::debug;
use wekan_common::artifact::common::{AType, SortedArtifact};

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
