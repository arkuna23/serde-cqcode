pub mod access;
pub mod core;

pub use core::CQDeserializer;

use serde::Deserialize;

use crate::{data::CQCode, Result};

pub fn from_cq_str(input: impl AsRef<str>) -> Result<Vec<CQCode>> {
    Deserialize::deserialize(&mut CQDeserializer::new(input.as_ref()))
}

mod test {

    #[test]
    fn test_de() {
        use super::*;

        assert_eq!(
            vec![
                CQCode::from(("text", [("text", "你好")])),
                CQCode::from(("face", [("id", "1")]))
            ],
            from_cq_str("你好[CQ:face,id=1]").unwrap()
        )
    }
}
