// 1. local modules
mod pos;

// 2. stdlib
use std::{
    collections::{HashMap, HashSet},
    env,
};

// 3. external crates
use harper_core::spell::{Dictionary, FstDictionary};
use serde::Deserialize;

// 4. local modules
use pos::Pos;

pub struct Cfg {
    pub raw: bool,
    pub alternatives: Vec<String>,
}

fn cli() -> Cfg {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let mut cfg = Cfg {
        raw: false,
        alternatives: Vec::new(),
    };

    while let Some(arg) = args.pop() {
        if arg == "--raw" {
            cfg.raw = true;
        } else {
            cfg.alternatives.push(arg);
        }
    }

    cfg
}

fn build_url(cfg: &Cfg) -> url::Url {
    let mut url = url::Url::parse("https://books.google.com/ngrams/json").unwrap();

    // comma-separated list of terms with wildcards
    let content = cfg
        .alternatives
        .iter()
        .map(|t| format!("* {},{} *", t, t))
        .collect::<Vec<_>>()
        .join(",");

    url.query_pairs_mut().append_pair("content", &content);

    url
}

fn fetch_json(url: url::Url) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;
    let json = response.json::<serde_json::Value>()?;
    Ok(json)
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
struct NgramItem {
    ngram: String,
    parent: String,
    #[serde(rename = "type")]
    kind: NgramType,
    // We may use this later with clustering to pay only attention to the highest frequency
    #[serde(rename = "timeseries")]
    _timeseries: serde_json::Value,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum NgramType {
    NgramCollection,
    Expansion,
    Ngram,
}

// A 'variant' is a word or phrase we're comparing against other variants: "their" vs "there" etc.
// A 'context' is a single word that appears before or after a variant: [their] "own" or [there] "is"
type VariantContextMap = HashMap<String, [Vec<String>; 2]>;

fn parse(json: serde_json::Value) -> Result<VariantContextMap, Box<dyn std::error::Error>> {
    let items: Vec<NgramItem> = serde_json::from_value(json)?;
    let mut variant_to_pre_and_post = VariantContextMap::new();

    for item in items {
        if item.kind != NgramType::Expansion {
            continue;
        }

        let (index, variant, context_word) = match item.parent.as_str() {
            p if p.starts_with("* ") => {
                let v = &p[2..];
                let c = item
                    .ngram
                    .strip_suffix(&format!(" {}", v))
                    .unwrap_or(&item.ngram);
                (0, v.to_string(), c.to_string())
            }
            p if p.ends_with(" *") => {
                let v = &p[..p.len() - 2];
                let c = item
                    .ngram
                    .strip_prefix(&format!("{} ", v))
                    .unwrap_or(&item.ngram);
                (1, v.to_string(), c.to_string())
            }
            _ => {
                return Err(format!(
                    "Parent '{}' must start with '* ' or end with ' *'",
                    item.parent
                )
                .into());
            }
        };

        variant_to_pre_and_post.entry(variant).or_default()[index].push(context_word);
    }

    Ok(variant_to_pre_and_post)
}

// The variants become the values, since we're looking for contexts that are unique to each variant
pub struct Analysis {
    pub pre_poses: HashMap<&'static Pos, HashSet<String>>,
    pub post_poses: HashMap<&'static Pos, HashSet<String>>,
    pub pre_words_case: HashMap<String, Vec<String>>,
    pub post_words_case: HashMap<String, Vec<String>>,
    pub pre_words_nocase: HashMap<String, Vec<String>>,
    pub post_words_nocase: HashMap<String, Vec<String>>,
}

// Look up part-of-speech tags for a word using Harper's curated dictionary
fn get_poses(dict: &FstDictionary, word: &str) -> Vec<&'static Pos> {
    dict.get_word_metadata_str(word)
        .map_or_else(std::vec::Vec::new, |md| {
            pos::POS_DEFINITIONS
                .iter()
                .filter(|&(_, pred)| pred(&md))
                .map(|(enum_variant, _)| enum_variant)
                .collect()
        })
}

fn analyse(
    variant_to_pre_and_post: VariantContextMap,
) -> Result<Analysis, Box<dyn std::error::Error>> {
    let dict = FstDictionary::curated();

    let mut res = Analysis {
        pre_poses: HashMap::new(),
        post_poses: HashMap::new(),
        pre_words_case: HashMap::new(),
        post_words_case: HashMap::new(),
        pre_words_nocase: HashMap::new(),
        post_words_nocase: HashMap::new(),
    };

    for (variant, [pre_words, post_words]) in variant_to_pre_and_post {
        for word in pre_words {
            res.pre_words_case
                .entry(word.clone())
                .or_default()
                .push(variant.clone());

            res.pre_words_nocase
                .entry(word.to_lowercase())
                .or_default()
                .push(variant.clone());

            for pos in get_poses(&dict, &word) {
                res.pre_poses
                    .entry(pos)
                    .or_default()
                    .insert(variant.clone());
            }
        }

        for word in post_words {
            res.post_words_case
                .entry(word.clone())
                .or_default()
                .push(variant.clone());

            res.post_words_nocase
                .entry(word.to_lowercase())
                .or_default()
                .push(variant.clone());

            for pos in get_poses(&dict, &word) {
                res.post_poses
                    .entry(pos)
                    .or_default()
                    .insert(variant.clone());
            }
        }
    }

    Ok(res)
}

fn output(cfg: Cfg, analysis: Analysis) {
    let Analysis {
        pre_poses,
        post_poses,
        pre_words_case,
        post_words_case,
        pre_words_nocase,
        post_words_nocase,
    } = analysis;

    for variant in &cfg.alternatives {
        let pre_pos = pre_poses
            .iter()
            .filter(|(_, poses)| poses.len() == 1 && poses.contains(variant))
            .map(|(pos, _)| pos)
            .collect::<Vec<_>>();

        let post_pos = post_poses
            .iter()
            .filter(|(_, poses)| poses.len() == 1 && poses.contains(variant))
            .map(|(pos, _)| pos)
            .collect::<Vec<_>>();

        let mut pre_words = pre_words_case
            .iter()
            .filter(|(_, variants)| variants.len() == 1 && variants.contains(variant))
            .map(|(word, _)| word.as_str())
            .collect::<Vec<_>>();
        pre_words.sort_by_key(|a| a.to_lowercase());

        let mut post_words = post_words_case
            .iter()
            .filter(|(_, variants)| variants.len() == 1 && variants.contains(variant))
            .map(|(word, _)| word.as_str())
            .collect::<Vec<_>>();
        post_words.sort_by_key(|a| a.to_lowercase());

        let mut pre_words_nocase = pre_words_nocase
            .iter()
            .filter(|(_, variants)| variants.len() == 1 && variants.contains(variant))
            .map(|(word, _)| word.as_str())
            .collect::<Vec<_>>();
        pre_words_nocase.sort_by_key(|a| a.to_lowercase());

        let mut post_words_nocase = post_words_nocase
            .iter()
            .filter(|(_, variants)| variants.len() == 1 && variants.contains(variant))
            .map(|(word, _)| word.as_str())
            .collect::<Vec<_>>();
        post_words_nocase.sort_by_key(|a| a.to_lowercase());

        // POS and case-sensitive word discriminators
        println!(
            "\x1b[35m{} | \x1b[36m{} \x1b[33m«« {} »» \x1b[34m{} \x1b[32m| {}\x1b[0m",
            pre_pos
                .iter()
                .map(|pos| pos::pos_info(pos).letter)
                .collect::<Vec<_>>()
                .join("/"),
            pre_words.join("|"),
            variant,
            post_words.join("|"),
            post_pos
                .iter()
                .map(|pos| pos::pos_info(pos).letter)
                .collect::<Vec<_>>()
                .join("/")
        );

        // Case-insensitive word discriminators
        println!(
            "    \x1b[36m{} \x1b[33m«« {} »» \x1b[34m{} \x1b[0m",
            pre_words_nocase.join("|"),
            variant,
            post_words_nocase.join("|"),
        );

        // Negative POS and case-sensitive word discriminators
        // appears with all variants but this one
        if cfg.alternatives.len() > 2 {
            let mut negative_pre_words = pre_words_case
                .iter()
                .filter(|(_, variants)| {
                    variants.len() == cfg.alternatives.len() - 1 && !variants.contains(variant)
                })
                .map(|(word, _)| word.as_str())
                .collect::<Vec<_>>();
            negative_pre_words.sort_by_key(|a| a.to_lowercase());

            let mut negative_post_words = post_words_case
                .iter()
                .filter(|(_, variants)| {
                    variants.len() == cfg.alternatives.len() - 1 && !variants.contains(variant)
                })
                .map(|(word, _)| word.as_str())
                .collect::<Vec<_>>();
            negative_post_words.sort_by_key(|a| a.to_lowercase());

            println!(
                "🚫 \x1b[31m {} | \x1b[33m{} \x1b[31m| {} \x1b[0m",
                negative_pre_words.join("|"),
                variant,
                negative_post_words.join("|")
            );
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = cli();
    let url = build_url(&cfg);
    let mut graph_url = url.clone();
    graph_url.path_segments_mut().unwrap().pop().push("graph");
    println!("ℹ️ URL: {}", graph_url);

    let variant_to_pre_and_post = parse(fetch_json(url)?)?;

    let analysis = analyse(variant_to_pre_and_post)?;

    output(cfg, analysis);

    Ok(())
}
