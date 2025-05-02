# serde-cqcode

English | [中文](./README.md)

`serde-cqcode` is a Rust library designed to serialize and deserialize CQ codes, which are commonly used in messaging applications to represent rich media content like images, emojis, and mentions. This library leverages the `serde` framework to provide a seamless way to handle CQ codes in Rust applications.

## Features

- **Serialization and Deserialization**: Easily convert CQ codes to and from strings using Serde.
- **Flexible Data Representation**: Allows for the representation of CQ codes as structured data, making it easy to manipulate and inspect.
- **Integration with Serde**: Seamlessly integrates with the Serde framework, enabling the use of familiar serialization and deserialization patterns.

## Installation

Add `serde-cqcode` to your `Cargo.toml`:

```toml
[dependencies]
serde-cqcode = { git = "https://github.com/arkuna23/serde-cqcode.git" }

#...
```


## Usage

Here's a basic example of how to use `serde-cqcode` to serialize and deserialize CQ codes:

```rust
use serde_cqcode::{from_str, to_string, CQCode};

fn main() {
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
        "你好[CQ:face,id=1][CQ:image,file=example.jpg][CQ:at,qq=123456][CQ:emoji,id=128512]";
    let expected = vec![
        Segment::Text {
            text: "你好".to_string(),
        },
        Segment::Face { id: 1 },
        Segment::Image {
            file: "example.jpg".to_string(),
        },
        Segment::At { qq: 123456 },
        Segment::Emoji { id: 128512 },
    ];

    let deserialized: Vec<Segment> = from_str(input).unwrap();
    assert_eq!(deserialized, expected);

    let serialized = to_string(&expected).unwrap();
    assert_eq!(serialized, input);
}
```

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.

## Disclaimer

This project is intended for learning purposes only. It is **not recommended** for use in production environments.

## License

This project is licensed under the MIT License.

## Contact

For any questions or suggestions, please open an issue on the GitHub repository.
