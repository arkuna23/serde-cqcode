pub(crate) mod access;
pub(crate) mod core;

pub use core::CQDeserializer;

use serde::Deserialize;

use crate::Result;

/// Deserializes a string slice into a data structure of type `T`.
///
/// # Arguments
///
/// * `input` - A string slice that holds the data to be deserialized.
///
/// # Returns
///
/// * `Result<T>` - Returns a result containing the deserialized data structure of type `T` on success,
///   or an error if the deserialization fails.
///
/// # Type Parameters
///
/// * `T` - The type of the data structure to deserialize into. It must implement the `Deserialize` trait.
pub fn from_str<'de, T: Deserialize<'de>>(input: &'de str) -> Result<T> {
    Deserialize::deserialize(&mut CQDeserializer::new(input))
}

mod test {
    #[test]
    fn test_de() {
        use super::*;
        use crate::data::CQCode;

        assert_eq!(
            vec![
                CQCode::from(("text", [("text", "[你好]")])),
                CQCode::from(("face", [("id", "1")])),
                CQCode::from(("image", [("file", "example.jpg")])),
                CQCode::from(("at", [("qq", "123456")])),
                CQCode::from(("emoji", [("id", "128512")]))
            ],
            from_str::<Vec<CQCode>>("&#91;你好&#93;[CQ:face,id=1][CQ:image,file=example.jpg][CQ:at,qq=123456][CQ:emoji,id=128512]").unwrap());
    }

    #[test]
    fn test_de_enum() {
        use super::*;

        #[derive(Deserialize, Debug, PartialEq)]
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

        let result: Vec<Segment> = from_str(input).unwrap();
        assert_eq!(result, expected);
    }
}
