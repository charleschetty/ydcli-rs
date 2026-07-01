//! 数据模型：释义、发音、词形变化、例句

use crate::color::{styled, Color};
use crate::html::{find_tags, strip_html};

// ── Paraphrase ───────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct Paraphrase {
    pub(crate) entries: Vec<String>,
}

impl Paraphrase {
    #[must_use]
    pub fn parse(html: &str) -> Self {
        // 英文页面：<li> 元素
        let entries: Vec<String> = find_tags(html, "div", "#phrsListTab")
            .iter()
            .flat_map(|section| find_tags(section, "div", ".trans-container"))
            .flat_map(|section| find_tags(section, "ul", ""))
            .flat_map(|ul| find_tags(ul, "li", ""))
            .map(strip_html)
            .filter(|s| !s.is_empty())
            .collect();
        if !entries.is_empty() {
            return Self { entries };
        }
        // 中文页面：<p class="wordGroup"> 内含 contentTitle 链接
        let entries: Vec<String> = find_tags(html, "div", "#phrsListTab")
            .iter()
            .flat_map(|section| find_tags(section, "div", ".trans-container"))
            .flat_map(|section| find_tags(section, "p", ".wordGroup"))
            .flat_map(|wg| find_tags(wg, "span", ".contentTitle"))
            .map(|ct| {
                // contentTitle 内有 <a href="...">word</a>，也接受裸文字
                let a_text: String = find_tags(ct, "a", "")
                    .iter()
                    .map(|a| strip_html(a))
                    .collect::<Vec<_>>()
                    .join("; ");
                if a_text.is_empty() { strip_html(ct) } else { a_text }
            })
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
    pub(crate) entries: Vec<PronounceEntry>,
}

impl Pronounce {
    #[must_use]
    pub fn parse(html: &str) -> Self {
        // 英文页面：<span class="pronounce"> 内含 英/美 音标
        let entries: Vec<PronounceEntry> = find_tags(html, "span", ".pronounce")
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
        if !entries.is_empty() {
            return Self { entries };
        }
        // 中文页面：<span class="phonetic">[nǐ hǎo]</span>（单个音标，无地区区分）
        if let Some(phonetic) = find_tags(html, "span", ".phonetic")
            .first()
            .map(|inner| strip_html(inner))
        {
            if !phonetic.is_empty() {
                return Self {
                    entries: vec![PronounceEntry {
                        region: String::new(),
                        phonetic,
                    }],
                };
            }
        }
        Self { entries: vec![] }
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
    pub(crate) forms: Vec<String>,
}

impl Variant {
    #[must_use]
    pub fn parse(html: &str) -> Self {
        // 提取 #phrsListTab .trans-container 内的 <p>，过滤掉 wordGroup 类（中文翻译）
        let forms = find_tags(html, "div", "#phrsListTab")
            .iter()
            .flat_map(|section| find_tags(section, "div", ".trans-container"))
            .flat_map(|section| {
                // 手动扫描 <p> 标签，跳过带 wordGroup 类的
                let mut out = Vec::new();
                let mut pos = 0;
                while let Some(start) = section[pos..].find("<p") {
                    let abs = pos + start;
                    let tag_end = section[abs..].find('>').map(|i| abs + i).unwrap_or(section.len());
                    let open_tag = &section[abs..=tag_end];
                    if open_tag.contains("wordGroup") {
                        pos = tag_end + 1;
                        continue;
                    }
                    if let Some(inner) = find_tags(&section[abs..], "p", "").first() {
                        out.push(strip_html(inner));
                    }
                    pos = tag_end + 1;
                }
                out
            })
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

pub(crate) type SentenceGroup = Vec<String>;

#[derive(Debug)]
pub(crate) struct SentenceElement {
    groups: Vec<SentenceGroup>,
    meaning: String,
    head: String,
}

impl SentenceElement {
    fn output(&self, num: usize) {
        // 有头部信息时才打印标题行
        if !self.head.is_empty() || !self.meaning.is_empty() {
            println!(
                "{num}. {}{}",
                styled(&self.head, Color::Green),
                styled(&self.meaning, Color::Reset),
            );
        }
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
    pub(crate) entries: Vec<SentenceElement>,
}

impl Example {
    #[must_use]
    pub fn parse(html: &str) -> Self {
        // 英文页面：.collinsToggle li
        let mut entries = Self::parse_collins(html);
        // 中文页面：回退到 #bilingual li
        if entries.is_empty() {
            entries = Self::parse_bilingual(html);
        }
        Self { entries }
    }

    /// 英文例句（Collins 词典）
    fn parse_collins(html: &str) -> Vec<SentenceElement> {
        let mut entries = Vec::new();

        for li in find_tags(html, "div", ".collinsToggle")
            .iter()
            .flat_map(|section| find_tags(section, "li", ""))
        {
            let head: Vec<String> = find_tags(li, "span", ".additional")
                .iter()
                .map(|h| strip_html(h))
                .filter(|s| !s.is_empty())
                .collect();
            if head.is_empty() {
                continue;
            }

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

            let groups: Vec<SentenceGroup> = find_tags(li, "div", ".examples")
                .iter()
                .map(|div| {
                    find_tags(div, "p", "")
                        .iter()
                        .map(|p| strip_html(p))
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .filter(|g: &Vec<String>| !g.is_empty())
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

        entries
    }

    /// 中文例句（#bilingual 区域）
    fn parse_bilingual(html: &str) -> Vec<SentenceElement> {
        let mut entries = Vec::new();

        for li in find_tags(html, "div", "#bilingual")
            .iter()
            .flat_map(|section| find_tags(section, "li", ""))
        {
            let sentences: Vec<String> = find_tags(li, "p", "")
                .iter()
                .map(|p| strip_html(p))
                .filter(|s| !s.is_empty())
                .collect();
            if sentences.is_empty() {
                continue;
            }

            entries.push(SentenceElement {
                groups: vec![sentences],
                meaning: String::new(),
                head: String::new(),
            });
        }

        entries
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

    const EMPTY: &str = "<html></html>";

    #[test]
    fn empty_html() {
        assert!(Paraphrase::parse(EMPTY).is_empty());
        assert!(Pronounce::parse(EMPTY).is_empty());
        assert!(Variant::parse(EMPTY).is_empty());
        assert!(Example::parse(EMPTY).is_empty());
    }

    #[test]
    fn paraphrase() {
        let html = r#"<div id="phrsListTab"><div class="trans-container"><ul>
            <li>世界；地球；天下</li><li>领域；界</li>
        </ul></div></div>"#;
        let p = Paraphrase::parse(html);
        assert_eq!(p.entries.len(), 2);
        assert_eq!(p.entries[0], "世界；地球；天下");
    }

    #[test]
    fn pronounce() {
        let html = r#"<span class="pronounce"><span>英</span><span>/wɜːld/</span></span>
        <span class="pronounce"><span>美</span><span>/wɜːrld/</span></span>"#;
        let p = Pronounce::parse(html);
        assert_eq!(p.entries.len(), 2);
        assert_eq!(p.entries[0].region, "英");
        assert_eq!(p.entries[0].phonetic, "/wɜːld/");
    }

    #[test]
    fn variant() {
        let html = r#"<div id="phrsListTab"><div class="trans-container">
            <p>复数：worlds</p></div></div>"#;
        assert_eq!(Variant::parse(html).forms.len(), 1);
    }

    #[test]
    fn example_basic() {
        let html = r#"<div class="collinsToggle"><li>
            <span class="additional">N-SING</span>
            <div class="collinsMajorTrans"><p>The world is the planet</p></div>
            <div class="examples"><p>It's a small world.</p></div>
        </li></div>"#;
        assert!(!Example::parse(html).is_empty());
    }
}
