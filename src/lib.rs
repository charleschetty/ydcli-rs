//! ydcli-rs — 有道词典命令行版 （零依赖）
//!
//! 从 [dict.youdao.com](http://dict.youdao.com) 抓取单词释义、发音、词形变化和例句，
//! 并以带 ANSI 颜色的格式输出到终端。纯标准库实现。

use std::io::{Read, Write};
use std::net::TcpStream;

/// 便捷的错误类型别名
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// ── HTTP ──────────────────────────────────────────────────────────────────

/// 用原生 TCP 发送 HTTP GET 请求，返回响应体
fn http_get(host: &str, path: &str) -> Result<String> {
    let mut stream = TcpStream::connect((host, 80))
        .map_err(|e| format!("无法连接到 {host}：{e}"))?;

    let request = format!("GET {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("发送请求失败：{e}"))?;

    let mut buf = Vec::new();
    stream
        .read_to_end(&mut buf)
        .map_err(|e| format!("读取响应失败：{e}"))?;

    let response = String::from_utf8_lossy(&buf);
    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("");
    Ok(body.to_owned())
}

/// 从有道词典抓取单词页面的 HTML
pub fn fetch_word(word: &str) -> Result<String> {
    let path = format!("/search?q={word}");
    http_get("dict.youdao.com", &path)
}

// ── ANSI Color ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Cyan,
    Gray,
    Magenta,
    Bold,
    Reset,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Red => write!(f, "\x1b[31m"),
            Color::Green => write!(f, "\x1b[32m"),
            Color::Yellow => write!(f, "\x1b[33m"),
            Color::Blue => write!(f, "\x1b[34m"),
            Color::Cyan => write!(f, "\x1b[36m"),
            Color::Gray => write!(f, "\x1b[38;5;8m"),
            Color::Magenta => write!(f, "\x1b[35m"),
            Color::Bold => write!(f, "\x1b[1m"),
            Color::Reset => write!(f, "\x1b[0m"),
        }
    }
}

/// 用 ANSI 颜色包裹文本
pub fn styled(text: &str, color: Color) -> String {
    format!("{color}{text}{reset}", color = color, reset = Color::Reset)
}

// ── HTML 解析引擎 ────────────────────────────────────────────────────────

/// 检查标签的 class 属性中是否包含指定值
fn tag_has_class(open_tag: &str, class: &str) -> bool {
    let mut pos = 0;
    while let Some(start) = open_tag[pos..].find("class=\"") {
        let val_start = pos + start + 7; // skip 'class="'
        if let Some(val_end) = open_tag[val_start..].find('"') {
            let values = &open_tag[val_start..val_start + val_end];
            if values.split_whitespace().any(|c| c == class) {
                return true;
            }
            pos = val_start + val_end + 1;
        } else {
            break;
        }
    }
    false
}

/// 检查标签的 id 属性是否匹配
fn tag_has_id(open_tag: &str, id: &str) -> bool {
    open_tag.contains(&format!("id=\"{id}\""))
        || open_tag.contains(&format!("id='{id}'"))
}

/// 检查开标签是否匹配属性提示：
/// - `.classname` → 匹配 class 属性
/// - `#id` → 匹配 id 属性
/// - 裸字符串 → 原始子串匹配
fn attr_matches(open_tag: &str, hint: &str) -> bool {
    if let Some(class) = hint.strip_prefix('.') {
        tag_has_class(open_tag, class)
    } else if let Some(id) = hint.strip_prefix('#') {
        tag_has_id(open_tag, id)
    } else {
        open_tag.contains(hint)
    }
}

/// 在 HTML 字符串中查找指定标签的内容。
/// `attr_hint`: `.class` 匹配 class, `#id` 匹配 id, 裸字符串做子串匹配。
/// 能正确处理同标签嵌套
fn find_tags<'a>(html: &'a str, tag: &str, attr_hint: &str) -> Vec<&'a str> {
    let mut results = Vec::new();
    let open_marker = format!("<{tag}");
    let close_marker = format!("</{tag}>");

    let mut pos = 0;
    let bytes = html.as_bytes();

    while pos < bytes.len() {
        // 找到下一个 <tag... 出现位置
        let rest = &bytes[pos..];
        let found = rest
            .windows(open_marker.len())
            .position(|w| w == open_marker.as_bytes());

        let tag_start = match found {
            Some(i) => pos + i,
            None => break,
        };

        // 找到开标签的结束位置 '>'
        let tag_end = match bytes[tag_start..].iter().position(|&b| b == b'>') {
            Some(i) => tag_start + i,
            None => break,
        };

        let open_tag = &html[tag_start..=tag_end];

        // 跳过不被期望的属性提示匹配的标签
        if attr_hint.is_empty() || attr_matches(open_tag, attr_hint) {
            // 自闭合标签？
            if bytes[tag_end - 1] == b'/' {
                results.push("");
                pos = tag_end + 1;
                continue;
            }

            // 数嵌套深度找到匹配的闭合标签
            let mut depth = 1u32;
            let mut search = tag_end + 1;

            while depth > 0 && search < bytes.len() {
                let tail = &bytes[search..];

                // 找下一个 <tag 或 </tag>
                let next_open = tail
                    .windows(open_marker.len())
                    .position(|w| w == open_marker.as_bytes());
                let next_close = tail
                    .windows(close_marker.len())
                    .position(|w| w == close_marker.as_bytes());

                // close 优先于 open（当 close <= open 时先处理 close）
                let close_first = match (next_open, next_close) {
                    (None, Some(_)) => true,
                    (Some(o), Some(c)) => c <= o,
                    _ => false,
                };

                if close_first {
                    if let Some(ci) = next_close {
                        depth -= 1;
                        if depth == 0 {
                            let inner = &html[tag_end + 1..search + ci];
                            results.push(inner);
                            pos = search + ci + close_marker.len();
                            break;
                        }
                        search += ci + close_marker.len();
                    }
                } else if let Some(oi) = next_open {
                    depth += 1;
                    search += oi + open_marker.len();
                } else {
                    break; // 既无开标签也无闭标签，格式错误的 HTML
                }
            }
            if depth == 0 {
                continue;
            }
        }

        pos = tag_end + 1;
    }

    results
}

/// 去除 HTML 标签，解码常见 HTML 实体，合并空白。
/// 每个标签被替换为一个空格，以分隔相邻的文本节点。
fn strip_html(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut in_tag = false;
    for ch in raw.chars() {
        if ch == '<' {
            in_tag = true;
            out.push(' '); // 用空格替代标签，分隔相邻文本
            continue;
        }
        if ch == '>' {
            in_tag = false;
            continue;
        }
        if !in_tag {
            out.push(ch);
        }
    }
    // 解码常见实体
    let out = out
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");
    // 合并多余空白
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ── 输出入口 ─────────────────────────────────────────────────────────────

/// 输出所有查询结果到终端
pub fn output_all(html: &str, word: &str) {
    println!("{}", styled(word, Color::Red));

    Pronounce::parse(html).output();
    Paraphrase::parse(html).output();
    Variant::parse(html).output();
    println!();
    Example::parse(html).output();
}

// ── Paraphrase ───────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct Paraphrase {
    entries: Vec<String>,
}

impl Paraphrase {
    #[must_use]
    pub fn parse(html: &str) -> Self {
        let entries = find_tags(html, "div", "#phrsListTab")
            .iter()
            .flat_map(|section| find_tags(section, "div", ".trans-container"))
            .flat_map(|section| find_tags(section, "ul", ""))
            .flat_map(|ul| find_tags(ul, "li", ""))
            .map(strip_html)
            .filter(|s| !s.is_empty())
            .collect();
        Self { entries }
    }

    pub fn output(&self) {
        for entry in &self.entries {
            println!("{entry}");
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ── Pronounce ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PronounceEntry {
    pub region: String,
    pub phonetic: String,
}

#[derive(Debug, Default)]
pub struct Pronounce {
    entries: Vec<PronounceEntry>,
}

impl Pronounce {
    #[must_use]
    pub fn parse(html: &str) -> Self {
        let entries = find_tags(html, "span", ".pronounce")
            .iter()
            .filter_map(|inner| {
                let text = strip_html(inner);
                let parts: Vec<&str> = text.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some(PronounceEntry {
                        region: parts[0].to_owned(),
                        phonetic: parts[1].to_owned(),
                    })
                } else {
                    None
                }
            })
            .collect();
        Self { entries }
    }

    pub fn output(&self) {
        if self.is_empty() {
            return;
        }
        for entry in &self.entries {
            print!(
                "{} {}  ",
                styled(&entry.region, Color::Reset),
                styled(&entry.phonetic, Color::Cyan),
            );
        }
        println!();
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ── Variant ──────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct Variant {
    forms: Vec<String>,
}

impl Variant {
    #[must_use]
    pub fn parse(html: &str) -> Self {
        let forms = find_tags(html, "div", "#phrsListTab")
            .iter()
            .flat_map(|section| find_tags(section, "div", ".trans-container"))
            .flat_map(|section| find_tags(section, "p", ""))
            .map(strip_html)
            .filter(|s| !s.is_empty())
            .collect();
        Self { forms }
    }

    pub fn output(&self) {
        for form in &self.forms {
            println!("{}", styled(form, Color::Cyan));
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.forms.is_empty()
    }
}

// ── Example ──────────────────────────────────────────────────────────────

/// 每个 `.examples` div 内的句子编为一组。
/// 每个组的第一句带 `例:` 前缀，同组后续句子只缩进。
type SentenceGroup = Vec<String>;

#[derive(Debug)]
struct SentenceElement {
    groups: Vec<SentenceGroup>,
    meaning: String,
    head: String,
}

impl SentenceElement {
    fn output(&self, num: usize) {
        println!(
            "{num}. {}{}",
            styled(&self.head, Color::Green),
            styled(&self.meaning, Color::Reset),
        );
        for group in &self.groups {
            for (i, sentence) in group.iter().enumerate() {
                let prefix = if i == 0 { "  例: " } else { "       " };
                println!(
                    "{}{}",
                    styled(prefix, Color::Green),
                    styled(sentence, Color::Yellow),
                );
            }
        }
        println!();
    }
}

#[derive(Debug, Default)]
pub struct Example {
    entries: Vec<SentenceElement>,
}

impl Example {
    #[must_use]
    pub fn parse(html: &str) -> Self {
        let mut entries = Vec::new();

        for li in find_tags(html, "div", ".collinsToggle")
            .iter()
            .flat_map(|section| find_tags(section, "li", ""))
        {
            // 解析头部 .additional
            let head: Vec<String> = find_tags(li, "span", ".additional")
                .iter()
                .map(|h| strip_html(h))
                .filter(|s| !s.is_empty())
                .collect();
            if head.is_empty() {
                continue;
            }

            // 解析释义 .collinsMajorTrans p
            let meaning_all: Vec<String> = find_tags(li, "div", ".collinsMajorTrans")
                .iter()
                .flat_map(|div| find_tags(div, "p", ""))
                .map(strip_html)
                .filter(|s| !s.is_empty())
                .collect();
            if meaning_all.is_empty() {
                continue;
            }

            let meaning = meaning_all[0]
                .strip_prefix(&head[0])
                .unwrap_or(&meaning_all[0])
                .to_owned();

            // 解析例句：每个 .examples div 为一组，组内第一句带 "例:" 前缀
            let groups: Vec<SentenceGroup> = find_tags(li, "div", ".examples")
                .iter()
                .map(|div| {
                    find_tags(div, "p", "")
                        .iter()
                        .map(|p| strip_html(p))
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<String>>()
                })
                .filter(|g| !g.is_empty())
                .collect();
            if groups.is_empty() {
                continue;
            }

            let head_text = if head.len() == 1 {
                format!("[{}]", head[0])
            } else {
                head.into_iter().nth(1).unwrap_or_default()
            };

            entries.push(SentenceElement {
                groups,
                meaning,
                head: head_text,
            });
        }

        Self { entries }
    }

    pub fn output(&self) {
        for (i, entry) in self.entries.iter().enumerate() {
            entry.output(i + 1);
        }
        println!("{}", Color::Reset);
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY_HTML: &str = "<html></html>";

    #[test]
    fn empty_html_parsing() {
        assert!(Paraphrase::parse(EMPTY_HTML).is_empty());
        assert!(Pronounce::parse(EMPTY_HTML).is_empty());
        assert!(Variant::parse(EMPTY_HTML).is_empty());
        assert!(Example::parse(EMPTY_HTML).is_empty());
    }

    #[test]
    fn parse_paraphrase() {
        let html = r#"<div id="phrsListTab"><div class="trans-container"><ul>
            <li>世界；地球；天下</li>
            <li>领域；界</li>
        </ul></div></div>"#;
        let p = Paraphrase::parse(html);
        assert_eq!(p.entries.len(), 2);
        assert_eq!(p.entries[0], "世界；地球；天下");
    }

    #[test]
    fn parse_pronounce() {
        let html = r#"<span class="pronounce"><span>英</span><span>/wɜːld/</span></span>
        <span class="pronounce"><span>美</span><span>/wɜːrld/</span></span>"#;
        let p = Pronounce::parse(html);
        assert_eq!(p.entries.len(), 2);
        assert_eq!(p.entries[0].region, "英");
        assert_eq!(p.entries[0].phonetic, "/wɜːld/");
        assert_eq!(p.entries[1].region, "美");
        assert_eq!(p.entries[1].phonetic, "/wɜːrld/");
    }

    #[test]
    fn parse_variant() {
        let html = r#"<div id="phrsListTab"><div class="trans-container">
            <p>复数：worlds</p>
        </div></div>"#;
        let v = Variant::parse(html);
        assert_eq!(v.forms.len(), 1);
    }

    #[test]
    fn parse_example_basic() {
        let html = r#"<div class="collinsToggle"><li>
            <span class="additional">N-SING</span>
            <div class="collinsMajorTrans"><p>The world is the planet</p></div>
            <div class="examples"><p>It's a small world.</p></div>
        </li></div>"#;
        let e = Example::parse(html);
        assert!(!e.is_empty());
    }

    #[test]
    fn strip_html_removes_tags_and_decodes() {
        assert_eq!(strip_html("<p>hello &amp; world</p>"), "hello & world");
        assert_eq!(strip_html("  <b>bold</b>  "), "bold");
        assert_eq!(strip_html("a&#39;b"), "a'b");
    }

    #[test]
    fn find_tags_nested_divs() {
        let html = "<div class=\"a\"><div class=\"b\">inner</div></div>";
        let outer = find_tags(html, "div", ".a");
        assert_eq!(outer.len(), 1);
        assert!(outer[0].contains("class=\"b\""));
    }

    #[test]
    fn color_display() {
        assert_eq!(
            format!("{}hello{}", Color::Red, Color::Reset),
            "\x1b[31mhello\x1b[0m"
        );
    }

    #[test]
    fn styled_works() {
        assert_eq!(styled("hi", Color::Green), "\x1b[32mhi\x1b[0m");
    }
}
