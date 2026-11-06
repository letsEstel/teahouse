//! 这个代码包里没有程序，只有书。下面每一行把一个小节交给 rustdoc，
//! 小节里的 Rust 代码就成了测试：`cargo test --manifest-path book/Cargo.toml`。

#[doc = include_str!("sections/how-to-read.md")]
pub mod how_to_read {}
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
