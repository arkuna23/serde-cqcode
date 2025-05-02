pub(crate) mod access;
pub(crate) mod core;

pub use core::CQDeserializer;

use serde::Deserialize;

use crate::Result;

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
                CQCode::from(("text", [("text", "你好")])),
                CQCode::from(("face", [("id", "1")])),
                CQCode::from(("image", [("file", "example.jpg")])),
                CQCode::from(("at", [("qq", "123456")])),
                CQCode::from(("emoji", [("id", "128512")]))
            ],
            from_str::<Vec<CQCode>>("你好[CQ:face,id=1][CQ:image,file=example.jpg][CQ:at,qq=123456][CQ:emoji,id=128512]").unwrap());
    }

    #[test]
    fn test_de_enum() {
        use super::*;

        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(rename_all = "lowercase")]
        enum Test {
            Text { text: String },
            Face { id: u32 },
            Image { file: String },
            At { qq: u64 },
            Emoji { id: u32 },
        }

        let input =
            "你好[CQ:face,id=1][CQ:image,file=example.jpg][CQ:at,qq=123456][CQ:emoji,id=128512]";
        let expected = vec![
            Test::Text {
                text: "你好".to_string(),
            },
            Test::Face { id: 1 },
            Test::Image {
                file: "example.jpg".to_string(),
            },
            Test::At { qq: 123456 },
            Test::Emoji { id: 128512 },
        ];

        let result: Vec<Test> = from_str(input).unwrap();
        assert_eq!(result, expected);
    }
}
