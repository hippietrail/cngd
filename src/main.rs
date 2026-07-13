mod clustering;

use std::{
    collections::HashMap,
    env, error,
    io::{self, BufRead},
};

#[macro_export]
macro_rules! vprintln {
    ($verbose:expr, $($arg:tt)*) => {
        if $verbose {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! evprintln {
    ($verbose:expr, $($arg:tt)*) => {
        if $verbose {
            eprintln!($($arg)*);
        }
    };
}

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

        vprintln!(
            verbose,
            "\x1b[34m{}\x1b[0m + \x1b[35m{}\x1b[0m \x1b[1;3mOR\x1b[0m \x1b[36m{}\x1b[0m + \x1b[37m{}\x1b[0m",
            first,
            maybe_core_end,
            maybe_core_start,
            last
        );

        if potential_cores.is_empty() {
            evprintln!(
                verbose,
                "Adding potential cores: {} and {}",
                maybe_core_end,
                maybe_core_start
            );
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

    if verbose {
        println!("📊 Frequency counts:");
        for (core, count) in sorted_cores.iter().take(8) {
            println!("  {}: {}", core, count);
        }
    }

    let core_cluster = clustering::extract_core_cluster(verbose, &sorted_cores);

    let targets: Vec<&str> = if !user_specified_cores.is_empty() {
        user_specified_cores.iter().map(|s| s.as_str()).collect()
    } else if auto_cluster {
        core_cluster
    } else {
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
                vprintln!(verbose, "  🍇 F‘{}’ X«{}»", target, post_context);
                post_context_map
                    .entry(post_context.to_string())
                    .or_default()
                    .push(target);
            }
            if let Some(pre_context) = line.strip_suffix(&target_suffix) {
                vprintln!(verbose, "  🍐 X«{}» F‘{}’", pre_context, target);
                pre_context_map
                    .entry(pre_context.to_string())
                    .or_default()
                    .push(target);
            }
        }
    }

    let mut confusable_contexts: HashMap<&str, (Vec<String>, Vec<String>)> = HashMap::new();
    let mut case_analysis: HashMap<String, Vec<String>> = HashMap::new();

    vprintln!(verbose, "Pre-context map:");
    for (context, confusables) in &pre_context_map {
        if confusables.len() == 1 {
            vprintln!(verbose, "  ‘{}’: {:?}", context, confusables);
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

    vprintln!(verbose, "Post-context map:");
    for (context, confusables) in &post_context_map {
        if confusables.len() == 1 {
            vprintln!(verbose, "  ‘{}’: {:?}", context, confusables);
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

    let count = confusable_contexts.len() as f32;
    println!("\n🎯 Confusable contexts:");

    for (i, (confusable, (pres, posts))) in confusable_contexts.iter().enumerate() {
        let hue = (i as f32 / count) * 360.0;

        let (r, g, b) = hsl_to_rgb(hue, 0.8, 0.2);

        println!(
            "\x1b[48;2;{};{};{}m ‘{}’: pre={:?}, post={:?} \x1b[0m",
            r, g, b, confusable, pres, posts
        );
    }

    vprintln!(verbose, "\n🔤 Case sensitivity analysis:");
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
            vprintln!(
                verbose,
                "  CASE-SENSITIVE: ‘{}’ (only: {:?})",
                lower,
                unique_variants
            );
            case_sensitive_contexts.push(lower);
        } else {
            vprintln!(
                verbose,
                "  CASE-INSENSITIVE: ‘{}’ (variants: {:?})",
                lower,
                unique_variants
            );
            case_insensitive_contexts.push(lower);
        }
    }

    vprintln!(
        verbose,
        "\n📋 Summary:\n\
          Case-sensitive contexts: {}\n\
          Case-insensitive contexts: {}",
        case_sensitive_contexts.len(),
        case_insensitive_contexts.len()
    );

    Ok(())
}

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
