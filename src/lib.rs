//! ydcli-rs — 有道词典命令行版（零依赖）
//!
//! 从 [dict.youdao.com](http://dict.youdao.com) 抓取单词释义、发音、词形变化和例句。
//! 纯标准库实现。

mod color;
mod html;
mod http;
mod model;

pub use color::{styled, Color};
pub use model::{Example, Paraphrase, Pronounce, PronounceEntry, Variant};

/// 便捷的错误类型别名
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// 从有道词典抓取单词页面（支持中英文）
pub fn fetch_word(word: &str) -> Result<String> {
    let path = format!("/search?q={}", http::url_encode(word));
    http::http_get("dict.youdao.com", &path)
}

/// 输出所有查询结果到终端
pub fn output_all(html: &str, word: &str) {
    println!("{}", styled(word, Color::Red));

    Pronounce::parse(html).output();
    Paraphrase::parse(html).output();
    Variant::parse(html).output();
    println!();
    Example::parse(html).output();
}
