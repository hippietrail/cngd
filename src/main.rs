mod pos;

use reqwest;
use serde_json;

use std::{
    collections::{HashMap, HashSet},
    env,
};

use harper_core::spell::{Dictionary, FstDictionary};

use pos::POS;

pub struct Cfg {
    // pub debug: bool,
    // pub verbose: bool,
    pub alternatives: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let mut cfg = Cfg {
        // debug: false,
        // verbose: false,
        alternatives: Vec::new(),
    };

    while let Some(arg) = args.pop() {
        // if arg == "--debug" {
        //     cfg.debug = true;
        // } else if arg == "--verbose" {
        //     cfg.verbose = true;
        // } else {
        cfg.alternatives.push(arg);
        // }
    }

    let mut url = url::Url::parse("https://books.google.com/ngrams/json").unwrap();

    // comma-separated list of terms with wildcards
    let content = cfg
        .alternatives
        .iter()
        .map(|t| format!("* {},{} *", t, t))
        .collect::<Vec<_>>()
        .join(",");

    url.query_pairs_mut().append_pair("content", &content);

    let mut graph_url = url.clone();
    graph_url.path_segments_mut().unwrap().pop().push("graph");
    println!("ℹ️ URL: {}", graph_url);

    let response = reqwest::blocking::get(url)?;
    let json = response.json::<serde_json::Value>()?;

    let mut variant_to_pre_and_post: HashMap<String, [Vec<String>; 2]> = HashMap::new();

    let known_keys = ["ngram", "parent", "timeseries", "type"];
    for ele in json.as_array().unwrap() {
        let ob = ele.as_object().unwrap();

        for key in ob.keys() {
            if !known_keys.contains(&key.as_str()) {
                return Err(format!("Unknown key: {}", key).into());
            }
            if key == "type" {
                let type_value = ob.get(key).unwrap();
                if type_value != "NGRAM_COLLECTION"
                    && type_value != "EXPANSION"
                    && type_value != "NGRAM"
                {
                    return Err(format!("Unknown type: {}", type_value).into());
                }
            }
        }

        let (ngram, parent, _timeseries, kind) = (
            ob.get("ngram").unwrap().as_str().unwrap(),
            ob.get("parent").unwrap().as_str().unwrap(),
            ob.get("timeseries").unwrap(),
            ob.get("type").unwrap().as_str().unwrap(),
        );

        enum Which {
            Start,
            End,
        }

        if kind == "EXPANSION" {
            let (wh, index) = match parent {
                p if p.starts_with("* ") => (Which::Start, 0),
                p if p.ends_with(" *") => (Which::End, 1),
                _ => {
                    return Err(format!(
                        "Parent {} does not start with '* ' or end with ' *'",
                        parent
                    )
                    .into());
                }
            };

            let (variant, context_word) = match wh {
                Which::Start => {
                    // parent is like "* to"
                    let v = parent.strip_prefix("* ").unwrap().to_string();
                    // ngram is like "according to" -> we want "according"
                    let c = ngram
                        .strip_suffix(&format!(" {}", v))
                        .unwrap_or(ngram) // Fallback safety
                        .to_string();
                    (v, c)
                }
                Which::End => {
                    // parent is like "to *"
                    let v = parent.strip_suffix(" *").unwrap().to_string();
                    // ngram is like "to make" -> we want "make"
                    let c = ngram
                        .strip_prefix(&format!("{} ", v))
                        .unwrap_or(ngram)
                        .to_string();
                    (v, c)
                }
            };

            variant_to_pre_and_post.entry(variant).or_default()[index].push(context_word);
        }
    }

    // for each variant, for each kind of context (pre, post), find which are the 'discriminators'
    // i.e. the ones in its set but not in the equivalent set of the other variants

    // a way to do this is to make a map from each context word to the variants that use it
    // then for each variant, for each kind of context, find the words that only appear in that variant's contexts

    let dict = FstDictionary::curated();

    let get_poses = |word: &str| -> Vec<&POS> {
        dict.get_word_metadata_str(word).map_or_else(
            || vec![],
            |md| {
                pos::POS_DEFINITIONS
                    .iter()
                    .filter(|&(_, pred)| pred(&md))
                    .map(|(enum_variant, _)| enum_variant)
                    .collect()
            },
        )
    };

    let mut pre_words_case: HashMap<String, Vec<String>> = HashMap::new();
    let mut post_words_case: HashMap<String, Vec<String>> = HashMap::new();

    let mut pre_words_nocase: HashMap<String, Vec<String>> = HashMap::new();
    let mut post_words_nocase: HashMap<String, Vec<String>> = HashMap::new();

    let mut pre_poses: HashMap<&POS, HashSet<String>> = HashMap::new();
    let mut post_poses: HashMap<&POS, HashSet<String>> = HashMap::new();

    let context_kind_names = ["pre", "post"];

    for (variant, contexts) in variant_to_pre_and_post.iter() {
        for (i, context_words_case) in contexts.iter().enumerate() {
            let context_kind = context_kind_names[i];
            let (ctx_to_case, ctx_to_nocase) = match context_kind {
                "pre" => (&mut pre_words_case, &mut pre_words_nocase),
                "post" => (&mut post_words_case, &mut post_words_nocase),
                _ => panic!("Invalid context kind name: {}", context_kind),
            };

            for context_word_case in context_words_case {
                ctx_to_case
                    .entry(context_word_case.clone())
                    .or_default()
                    .push(variant.clone());

                ctx_to_nocase
                    .entry(context_word_case.to_lowercase())
                    .or_default()
                    .push(variant.clone());

                for pos in get_poses(context_word_case) {
                    match context_kind {
                        "pre" => {
                            pre_poses.entry(pos).or_default().insert(variant.clone());
                        }
                        "post" => {
                            post_poses.entry(pos).or_default().insert(variant.clone());
                        }
                        _ => panic!("Invalid context kind name: {}", context_kind),
                    }
                }
            }
        }
    }

    output(
        cfg,
        pre_poses,
        post_poses,
        pre_words_case,
        post_words_case,
        pre_words_nocase,
        post_words_nocase,
    );

    Ok(())
}

fn output(
    cfg: Cfg,
    pre_poses: HashMap<&POS, HashSet<String>>,
    post_poses: HashMap<&POS, HashSet<String>>,
    pre_words_case: HashMap<String, Vec<String>>,
    post_words_case: HashMap<String, Vec<String>>,
    pre_words_nocase: HashMap<String, Vec<String>>,
    post_words_nocase: HashMap<String, Vec<String>>,
) {
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
            .filter(|(_, variants)| variants.len() == 1 && variants.contains(&variant))
            .map(|(word, _)| word.as_str())
            .collect::<Vec<_>>();
        pre_words.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        let mut post_words = post_words_case
            .iter()
            .filter(|(_, variants)| variants.len() == 1 && variants.contains(&variant))
            .map(|(word, _)| word.as_str())
            .collect::<Vec<_>>();
        post_words.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        let mut pre_words_nocase = pre_words_nocase
            .iter()
            .filter(|(_, variants)| variants.len() == 1 && variants.contains(&variant))
            .map(|(word, _)| word.as_str())
            .collect::<Vec<_>>();
        pre_words_nocase.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        let mut post_words_nocase = post_words_nocase
            .iter()
            .filter(|(_, variants)| variants.len() == 1 && variants.contains(&variant))
            .map(|(word, _)| word.as_str())
            .collect::<Vec<_>>();
        post_words_nocase.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

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
                    variants.len() == cfg.alternatives.len() - 1 && !variants.contains(&variant)
                })
                .map(|(word, _)| word.as_str())
                .collect::<Vec<_>>();
            negative_pre_words.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

            let mut negative_post_words = post_words_case
                .iter()
                .filter(|(_, variants)| {
                    variants.len() == cfg.alternatives.len() - 1 && !variants.contains(variant)
                })
                .map(|(word, _)| word.as_str())
                .collect::<Vec<_>>();
            negative_post_words.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

            println!(
                "🚫 \x1b[31m {} | \x1b[33m{} \x1b[31m| {} \x1b[0m",
                negative_pre_words.join("|"),
                variant,
                negative_post_words.join("|")
            );
        }
    }
}
