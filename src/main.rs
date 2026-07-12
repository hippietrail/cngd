mod clustering;

use std::{
    collections::HashMap,
    env, error,
    io::{self, BufRead},
};

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut verbose = false;
    let mut auto_cluster = false;
    let mut user_specified_cores: Vec<String> = Vec::new();

    for arg in env::args().skip(1) {
        if arg == "-v" || arg == "--verbose" {
            verbose = true;
        } else if arg == "--auto-cluster" {
            auto_cluster = true;
        } else {
            eprintln!("🤞: {}", arg);
            user_specified_cores.push(arg);
        }
    }

    let stdin = io::stdin();
    let lines: Vec<String> = stdin
        .lock()
        .lines()
        .map(|line| line.map(|l| l.trim().to_string()))
        .collect::<Result<_, _>>()?;

    let mut potential_cores: HashMap<&str, u32> = HashMap::new();

    for line in &lines {
        if line.is_empty() {
            return Err("Empty line encountered".into());
        }

        let (first, maybe_core_end) = line.split_once(' ').ok_or("Missing space separator")?;
        let (maybe_core_start, last) = line.rsplit_once(' ').ok_or("Missing space separator")?;

        if verbose {
            println!(
                "\x1b[34m{}\x1b[0m + \x1b[35m{}\x1b[0m \x1b[1;3mOR\x1b[0m \x1b[36m{}\x1b[0m + \x1b[37m{}\x1b[0m",
                first, maybe_core_end, maybe_core_start, last
            );
        }

        if potential_cores.is_empty() {
            if verbose {
                eprintln!(
                    "Adding potential cores: {} and {}",
                    maybe_core_end, maybe_core_start
                );
            }
            potential_cores.insert(maybe_core_end, 1);
            potential_cores.insert(maybe_core_start, 1);
            if potential_cores.len() != 2 {
                return Err("Potential cores are the same".into());
            }
        } else {
            *potential_cores.entry(maybe_core_end).or_insert(0) += 1;
            *potential_cores.entry(maybe_core_start).or_insert(0) += 1;
        }
    }

    let mut sorted_cores: Vec<_> = potential_cores.into_iter().collect();
    sorted_cores.sort_by(|a, b| b.1.cmp(&a.1));

    println!("📊 Frequency counts:");
    for (core, count) in sorted_cores.iter().take(8) {
        println!("  {}: {}", core, count);
    }

    let core_cluster = clustering::extract_core_cluster(&sorted_cores);

    // Determine our target core phrases dynamically
    let targets: Vec<&str> = if !user_specified_cores.is_empty() {
        // If the user passed terms manually via command line, use those
        user_specified_cores.iter().map(|s| s.as_str()).collect()
    } else if auto_cluster {
        // If --auto-cluster flag is set, use the dynamic list from clustering.rs
        core_cluster
    } else {
        // Default fallback behavior: take the top 2 elements
        if sorted_cores.len() < 2 {
            return Err("Not enough core data to extract default pairs.".into());
        }
        vec![sorted_cores[0].0, sorted_cores[1].0]
    };

    let mut pre_context_map: HashMap<String, Vec<&str>> = HashMap::new();
    let mut post_context_map: HashMap<String, Vec<&str>> = HashMap::new();

    for line in &lines {
        for &target in &targets {
            let target_prefix = format!("{} ", target);
            let target_suffix = format!(" {}", target);

            if let Some(post_context) = line.strip_prefix(&target_prefix) {
                if verbose {
                    println!("  🍇 F‘{}’ X«{}»", target, post_context);
                }
                post_context_map
                    .entry(post_context.to_string())
                    .or_default()
                    .push(target);
            }
            if let Some(pre_context) = line.strip_suffix(&target_suffix) {
                if verbose {
                    println!("  🍐 X«{}» F‘{}’", pre_context, target);
                }
                pre_context_map
                    .entry(pre_context.to_string())
                    .or_default()
                    .push(target);
            }
        }
    }

    let mut confusable_contexts: HashMap<&str, (Vec<String>, Vec<String>)> = HashMap::new();
    let mut case_analysis: HashMap<String, Vec<String>> = HashMap::new();

    if verbose {
        println!("Pre-context map:");
    }
    for (context, confusables) in &pre_context_map {
        if confusables.len() == 1 {
            if verbose {
                println!("  ‘{}’: {:?}", context, confusables);
            }
            let confusable = confusables[0];
            confusable_contexts
                .entry(confusable)
                .or_default()
                .0
                .push(context.clone());

            // Track case variations
            let lower = context.to_lowercase();
            case_analysis
                .entry(lower)
                .or_default()
                .push(context.clone());
        }
    }

    if verbose {
        println!("Post-context map:");
    }
    for (context, confusables) in &post_context_map {
        if confusables.len() == 1 {
            if verbose {
                println!("  ‘{}’: {:?}", context, confusables);
            }
            let confusable = confusables[0];
            confusable_contexts
                .entry(confusable)
                .or_default()
                .1
                .push(context.clone());

            // Track case variations
            let lower = context.to_lowercase();
            case_analysis
                .entry(lower)
                .or_default()
                .push(context.clone());
        }
    }

    // println!("\n🎯 Confusable contexts:");
    // for (confusable, (pres, posts)) in &confusable_contexts {
    //     println!("  ‘{}’: pre={:?}, post={:?}", confusable, pres, posts);
    // }
    let count = confusable_contexts.len() as f32;
    println!("\n🎯 Confusable contexts:");

    for (i, (confusable, (pres, posts))) in confusable_contexts.iter().enumerate() {
        // 1. Divide hue wheel by total count for maximally distinct colors
        let hue = (i as f32 / count) * 360.0;

        // 2. Convert HSL to 24-bit RGB (Fix L to ~0.2 for readable dark background)
        let (r, g, b) = hsl_to_rgb(hue, 0.8, 0.2);

        // 3. Print with 24-bit ANSI escape codes
        // 48;2;{r};{g};{b} sets the truecolor background, 39 resets text color to default
        println!(
            "\x1b[48;2;{};{};{}m ‘{}’: pre={:?}, post={:?} \x1b[0m",
            r, g, b, confusable, pres, posts
        );
    }

    if verbose {
        println!("\n🔤 Case sensitivity analysis:");
    }
    let mut case_sensitive_contexts: Vec<&str> = Vec::new();
    let mut case_insensitive_contexts: Vec<&str> = Vec::new();

    for (lower, variants) in &case_analysis {
        let unique_variants: Vec<_> = variants
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .cloned()
            .collect();
        if unique_variants.len() == 1 {
            if verbose {
                println!(
                    "  CASE-SENSITIVE: ‘{}’ (only: {:?})",
                    lower, unique_variants
                );
            }
            case_sensitive_contexts.push(lower);
        } else {
            if verbose {
                println!(
                    "  CASE-INSENSITIVE: ‘{}’ (variants: {:?})",
                    lower, unique_variants
                );
            }
            case_insensitive_contexts.push(lower);
        }
    }

    println!("\n📋 Summary:");
    println!(
        "  Case-sensitive contexts: {}",
        case_sensitive_contexts.len()
    );
    println!(
        "  Case-insensitive contexts: {}",
        case_insensitive_contexts.len()
    );

    Ok(())
}

// Helper function to convert HSL to RGB
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r1, g1, b1) = match h_prime as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r1 + m) * 255.0).round() as u8,
        ((g1 + m) * 255.0).round() as u8,
        ((b1 + m) * 255.0).round() as u8,
    )
}
