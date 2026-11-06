//! 这个代码包里没有程序，只有书。下面每一行把一个小节交给 rustdoc，
//! 小节里的 Rust 代码就成了测试：`cargo test --manifest-path book/Cargo.toml`。

#[doc = include_str!("sections/how-to-read.md")]
pub mod how_to_read {}
#[doc = include_str!("sections/0-overview/1-big-picture.md")]
pub mod s0 {}
#[doc = include_str!("sections/glossary.md")]
pub mod glossary {}
#[doc = include_str!("sections/1-start/1-terminal.md")]
pub mod s1_1 {}
#[doc = include_str!("sections/1-start/2-first-program.md")]
pub mod s1_2 {}
#[doc = include_str!("sections/1-start/3-names.md")]
pub mod s1_3 {}
#[doc = include_str!("sections/1-start/4-decisions.md")]
pub mod s1_4 {}
#[doc = include_str!("sections/1-start/5-functions.md")]
pub mod s1_5 {}
#[doc = include_str!("sections/2-rust/1-ownership.md")]
pub mod s2_1 {}
#[doc = include_str!("sections/2-rust/2-enums.md")]
pub mod s2_2 {}
#[doc = include_str!("sections/3-architecture/1-a-push.md")]
pub mod s3_1 {}
#[doc = include_str!("sections/3-architecture/2-your-own-hook.md")]
pub mod s3_2 {}
#[doc = include_str!("sections/3-architecture/3-what-it-defends.md")]
pub mod s3_3 {}
