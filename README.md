# serde-cqcode

`serde-cqcode` is a Rust library designed to serialize and deserialize CQ codes, which are commonly used in messaging applications to represent rich media content like images, emojis, and mentions. This library leverages the `serde` framework to provide a seamless way to handle CQ codes in Rust applications.

## Features

- **Serialization and Deserialization**: Easily convert CQ codes to and from strings using Serde.
- **Support for Common CQ Code Types**: Includes support for text, face, image, at, and emoji CQ codes.
- **Custom Error Handling**: Provides detailed error messages for unsupported types and parsing errors.

## Installation

Add `serde-cqcode` to your `Cargo.toml`:

```toml
[dependencies]
serde-cqcode = "0.1"

#...
```


## Usage

Here's a basic example of how to use `serde-cqcode` to serialize and deserialize CQ codes:

```rust
use serde_cqcode::{from_str, to_string, CQCode};

fn main() {
    let input = "你好[CQ:face,id=1][CQ:image,file=example.jpg][CQ:at,qq=123456][CQ:emoji,id=128512]";
    let codes: Vec<CQCode> = from_str(input).unwrap();

    for code in &codes {
        println!("{:?}", code);
    }

    let output = to_string(&codes).unwrap();
    println!("{}", output);
}
```

## Error Handling

`serde-cqcode` provides a custom `Error` type that implements `serde::de::Error` and `serde::ser::Error`. This allows for detailed error messages when serialization or deserialization fails.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.

## Disclaimer

This project is intended for learning purposes only. It is **not recommended** for use in production environments.

## License

This project is licensed under the MIT License.

## Contact

For any questions or suggestions, please open an issue on the GitHub repository.
