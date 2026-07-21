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

    let mut jurl = url::Url::parse("https://books.google.com/ngrams/json").unwrap();
    let mut gurl = url::Url::parse("https://books.google.com/ngrams/graph").unwrap();

    // comma-separated list of terms
    let content = cfg
        .alternatives
        .iter()
        .map(|t| format!("* {},{} *", t, t))
        .collect::<Vec<_>>()
        .join(",");

    jurl.query_pairs_mut().append_pair("content", &content);
    gurl.query_pairs_mut().append_pair("content", &content);

    println!("URL: {}", gurl);

    let response = reqwest::blocking::get(jurl)?;
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

    let mut pre_words_to_variants: HashMap<String, Vec<String>> = HashMap::new();
    let mut post_words_to_variants: HashMap<String, Vec<String>> = HashMap::new();

    let mut pre_poses_to_variants: HashMap<&POS, HashSet<String>> = HashMap::new();
    let mut post_poses_to_variants: HashMap<&POS, HashSet<String>> = HashMap::new();

    let context_kind_names = ["pre", "post"];

    for (variant, contexts) in variant_to_pre_and_post.iter() {
        for (i, context_words) in contexts.iter().enumerate() {
            let context_kind = context_kind_names[i];
            let ctx_to_variant = match context_kind {
                "pre" => &mut pre_words_to_variants,
                "post" => &mut post_words_to_variants,
                _ => panic!("Invalid context kind name: {}", context_kind),
            };
            for context_word in context_words {
                ctx_to_variant
                    .entry(context_word.clone())
                    .or_default()
                    .push(variant.clone());

                for pos in get_poses(context_word) {
                    match context_kind {
                        "pre" => {
                            pre_poses_to_variants
                                .entry(pos)
                                .or_default()
                                .insert(variant.clone());
                        }
                        "post" => {
                            post_poses_to_variants
                                .entry(pos)
                                .or_default()
                                .insert(variant.clone());
                        }
                        _ => panic!("Invalid context kind name: {}", context_kind),
                    }
                }
            }
        }
    }

    for variant in &cfg.alternatives {
        let pre_pos = pre_poses_to_variants
            .iter()
            .filter(|(_, poses)| poses.len() == 1 && poses.contains(variant))
            .map(|(pos, _)| pos)
            .collect::<Vec<_>>();
        let post_pos = post_poses_to_variants
            .iter()
            .filter(|(_, poses)| poses.len() == 1 && poses.contains(variant))
            .map(|(pos, _)| pos)
            .collect::<Vec<_>>();

        let pre_words = pre_words_to_variants
            .iter()
            .filter(|(_, variants)| variants.len() == 1 && variants.contains(&variant))
            .map(|(word, _)| word.as_str())
            .collect::<Vec<_>>();
        let post_words = post_words_to_variants
            .iter()
            .filter(|(_, variants)| variants.len() == 1 && variants.contains(&variant))
            .map(|(word, _)| word.as_str())
            .collect::<Vec<_>>();

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

        // Negative: appears with all-but-this-one variant
        if cfg.alternatives.len() > 2 {
            let negative_pre_words =
                pre_words_to_variants
                    .iter()
                    .filter(|(_, variants)| {
                        variants.len() == cfg.alternatives.len() - 1 
                        && !variants.contains(&variant)
                    })
                    .map(|(word, _)| word.as_str())
                    .collect::<Vec<_>>();

            let negative_post_words =
                post_words_to_variants
                    .iter()
                    .filter(|(_, variants)| {
                        variants.len() == cfg.alternatives.len() - 1 
                        && !variants.contains(variant)
                    })
                    .map(|(word, _)| word.as_str())
                    .collect::<Vec<_>>();

            println!(
                "🚫 \x1b[31m {} | \x1b[33m{} \x1b[31m| {} \x1b[0m",
                negative_pre_words.join("|"),
                variant,
                negative_post_words.join("|")
            );
        }
    }

    Ok(())
}
