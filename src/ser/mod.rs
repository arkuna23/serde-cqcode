use core::CQSerializer;

use serde::Serialize;

use crate::Result;

pub(crate) mod core;
pub(crate) mod util;

pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
    let mut ser = CQSerializer::default();
    value.serialize(&mut ser)?;
    Ok(ser.output)
}

mod test {

    #[test]
    fn test_ser() {
        use super::*;
        use crate::data::CQCode;

        assert_eq!(
            "你好[CQ:face,id=1][CQ:image,file=example.jpg][CQ:at,qq=123456][CQ:emoji,id=128512]",
            to_string(&[
                CQCode::from(("text", [("text", "你好")])),
                CQCode::from(("face", [("id", "1")])),
                CQCode::from(("image", [("file", "example.jpg")])),
                CQCode::from(("at", [("qq", "123456")])),
                CQCode::from(("emoji", [("id", "128512")])),
            ])
            .unwrap()
        )
    }
}
