use scraper::Selector;

use crate::tools::color::_CYAN;
use crate::tools::color::_GREEN;
use crate::tools::{
    color::{_RESET, _YELLOW},
    format_data, trim_str,
};

pub struct Paraphrase {
    paraphrase: Vec<String>,
}

impl Paraphrase {
    pub fn get(doc: &scraper::Html) -> Paraphrase {
        let _selector = Selector::parse("#phrsListTab .trans-container ul li").unwrap();
        let paraphrase = doc
            .select(&_selector)
            .map(|child| {
                child
                    .text()
                    .filter_map(trim_str)
                    .collect::<Vec<String>>()
                    .join("")
            })
            .collect();
        Paraphrase { paraphrase }
    }

    pub fn output(&self) {
        if self.is_empty() {
            return;
        }
        for element in &self.paraphrase {
            println!("{}", element);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.paraphrase.is_empty()
    }
}

pub struct Pronounce {
    pronounce: Vec<(String, String)>,
}

impl Pronounce {
    pub fn get(doc: &scraper::Html) -> Pronounce {
        let _selector = Selector::parse(" span.pronounce").unwrap();
        let pronounce: Vec<(String, String)> = doc
            .select(&_selector)
            .filter_map(|child| {
                let mut iter = child.text().filter_map(trim_str);
                match (iter.next(), iter.next()) {
                    (Some(region), Some(pron)) => Some((region, pron)),
                    _ => None,
                }
            })
            .collect();
        Pronounce { pronounce }
    }

    pub fn output(&self) {
        if self.is_empty() {
            return;
        }
        for element in &self.pronounce {
            print!(
                "{} {}  ",
                format_data(&element.0, _RESET),
                format_data(&element.1, _CYAN)
            );
        }
        print!("{}\n", format_data("", _RESET));
    }
    pub fn is_empty(&self) -> bool {
        self.pronounce.is_empty()
    }
}
struct SentenceElement {
    sentences: Vec<String>,
    meaning: String,
    head: String,
}

impl SentenceElement {
    fn output(&self, num: i32) {
        println!(
            "{}. {}{}",
            num,
            format_data(&self.head, _GREEN),
            format_data(&self.meaning, _RESET)
        );

        for item in &self.sentences {
            println!(
                "{}{}",
                format_data("  例: ", _GREEN),
                format_data(&item, _YELLOW)
            );
        }
        println!("{}", _RESET);
    }
}

pub struct Example {
    examples: Vec<SentenceElement>,
}

impl Example {
    pub fn get(doc: &scraper::Html) -> Example {
        let _selector = Selector::parse(".collinsToggle .ol li").unwrap();
        let _selector_head = Selector::parse(".additional").unwrap();
        let _selector_mean = Selector::parse(".collinsMajorTrans p").unwrap();
        let _selector_sentence = Selector::parse(".examples ").unwrap();

        let mut exmaples_vec: Vec<SentenceElement> = Vec::new();

        for element in doc.select(&_selector) {
            let head: Vec<String> = element
                .select(&_selector_head)
                .map(|child| {
                    child
                        .text()
                        .filter_map(trim_str)
                        .collect::<Vec<String>>()
                        .join("")
                })
                .collect();
            if head.len() == 0 {
                continue;
            }
            let len_of_head = head[0].len();

            let meaning_all: Vec<String> = element
                .select(&_selector_mean)
                .map(|child| {
                    child
                        .text()
                        .filter_map(trim_str)
                        .collect::<Vec<String>>()
                        .join("")
                })
                .collect();
            if meaning_all.len() ==0 {
                continue;
            }
            let len_of_meaning = meaning_all[0].len();
            let _meaning = &meaning_all[0][len_of_head..len_of_meaning];

            let examples: Vec<String> = element
                .select(&_selector_sentence)
                .map(|child| {
                    child
                        .text()
                        .filter_map(trim_str)
                        .collect::<Vec<String>>()
                        .join("")
                })
                .collect();
            if examples.len() == 0 {
                continue;
            }

            let head_fix: String = if head.len() == 1 {
                format!("[{}]", head[0].clone())
            } else {
                head[1].clone()
            };
            let res = SentenceElement {
                sentences: examples,
                meaning: _meaning.to_string(),
                head: head_fix,
            };
            exmaples_vec.push(res);
        }
        Example {
            examples: exmaples_vec,
        }
    }

    pub fn output(&self) {
        if self.is_empty() {
            return;
        }
        for i in 0..self.examples.len() {
            self.examples[i].output((i + 1) as i32);
        }
    }
    pub fn is_empty(&self) -> bool {
        self.examples.is_empty()
    }
}

pub struct Variant {
    rank: Vec<String>,
}

impl Variant {
    pub fn get(doc: &scraper::Html) -> Variant {
        let _selector = Selector::parse("#phrsListTab .trans-container p").unwrap();
        let rank = doc
            .select(&_selector)
            .flat_map(|child| {
                child.text().map(|t| {
                    t.split('\n')
                        .filter_map(trim_str)
                        .collect::<Vec<String>>()
                        .join(" ")
                })
            })
            .filter(|s| !s.is_empty())
            .collect();
        Variant { rank }
    }

    pub fn output(&self) {
        if self.is_empty() {
            return;
        }
        for element in &self.rank {
            println!("{}", format_data(&element, _CYAN));
        }
    }
    pub fn is_empty(&self) -> bool {
        self.rank.is_empty()
    }
}
