# serde-cqcode

[English](./README.en.md) | 中文

`serde-cqcode` 是一个 Rust 库，旨在序列化和反序列化 CQ 码，这些码通常用于消息应用中以表示丰富的媒体内容，如图像、表情符号和提及。该库利用 `serde` 框架提供了一种在 Rust 应用中处理 CQ 码的无缝方式。

## 特性

- **序列化和反序列化**：使用 Serde 轻松地将 CQ 码转换为字符串或从字符串转换。
- **灵活的数据表示**：允许将 CQ 码表示为结构化数据，便于操作和检查。
- **与 Serde 集成**：与 Serde 框架无缝集成，支持使用熟悉的序列化和反序列化模式。

## 安装

在你的 `Cargo.toml` 中添加 `serde-cqcode`：

```toml
[dependencies]
serde-cqcode = { git = "https://github.com/arkuna23/serde-cqcode.git" }

#...
```

## 用法

以下是如何使用 `serde-cqcode` 序列化和反序列化 CQ 码的基本示例：

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

## 贡献

欢迎贡献！请随时提交拉取请求或打开问题。

## 免责声明

此项目仅用于学习目的。**不建议**在生产环境中使用。

## 许可证

此项目根据 MIT 许可证授权。

## 联系

如有任何问题或建议，请在 GitHub 仓库上打开问题。
