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
            "&#91;你好&#93;[CQ:face,id=1][CQ:image,file=example.jpg][CQ:at,qq=123456][CQ:emoji,id=128512]",
            to_string(&[
                CQCode::from(("text", [("text", "[你好]")])),
                CQCode::from(("face", [("id", "1")])),
                CQCode::from(("image", [("file", "example.jpg")])),
                CQCode::from(("at", [("qq", "123456")])),
                CQCode::from(("emoji", [("id", "128512")])),
            ])
            .unwrap()
        )
    }

    #[test]
    fn test_enum_ser() {
        use super::*;
        use serde::*;

        #[derive(Serialize, Deserialize, Debug)]
        #[serde(rename_all = "lowercase")]
        enum Segment {
            Text { text: String },
            Face { id: u32 },
            Image { file: String },
            At { qq: u64 },
            Emoji { id: u32 },
        }

        let input =
            "&#91;你好&#93;[CQ:face,id=1][CQ:image,file=example.jpg][CQ:at,qq=123456][CQ:emoji,id=128512]";
        let expected = vec![
            Segment::Text {
                text: "[你好]".to_string(),
            },
            Segment::Face { id: 1 },
            Segment::Image {
                file: "example.jpg".to_string(),
            },
            Segment::At { qq: 123456 },
            Segment::Emoji { id: 128512 },
        ];

        let serialized = to_string(&expected).unwrap();
        assert_eq!(serialized, input);
    }
}
