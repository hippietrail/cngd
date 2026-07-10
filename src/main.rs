use std::{
    collections::HashMap,
    error,
    env,
    io::{self, BufRead},
};

fn main() -> Result<(), Box<dyn error::Error>> {
    let verbose = env::args().any(|arg| arg == "-v" || arg == "--verbose");
    
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

        let (first, maybe_core_end) = line
            .split_once(' ')
            .ok_or("Missing space separator")?;
        let (maybe_core_start, last) = line
            .rsplit_once(' ')
            .ok_or("Missing space separator")?;

        if verbose {
            println!(
                "\x1b[34m{}\x1b[0m + \x1b[35m{}\x1b[0m \x1b[1;3mOR\x1b[0m \x1b[36m{}\x1b[0m + \x1b[37m{}\x1b[0m",
                first, maybe_core_end, maybe_core_start, last
            );
        }

        if potential_cores.is_empty() {
            if verbose {
                eprintln!("Adding potential cores: {} and {}", maybe_core_end, maybe_core_start);
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

    let (confusable_1, confusable_2) = (&sorted_cores[0].0, &sorted_cores[1].0);

    let mut pre_context_map: HashMap<String, Vec<&str>> = HashMap::new();
    let mut post_context_map: HashMap<String, Vec<&str>> = HashMap::new();

    for line in &lines {
        let c1_space = format!("{} ", confusable_1);
        let c2_space = format!("{} ", confusable_2);
        let space_c1 = format!(" {}", confusable_1);
        let space_c2 = format!(" {}", confusable_2);

        if let Some(post_context) = line.strip_prefix(&c1_space) {
            if verbose {
                println!("  🍇 F‘{}’ X«{}»", confusable_1, post_context);
            }
            post_context_map.entry(post_context.to_string()).or_default().push(confusable_1);
        }
        if let Some(pre_context) = line.strip_suffix(&space_c2) {
            if verbose {
                println!("  🍐 X«{}» F‘{}’", pre_context, confusable_2);
            }
            pre_context_map.entry(pre_context.to_string()).or_default().push(confusable_2);
        }
        if let Some(post_context) = line.strip_prefix(&c2_space) {
            if verbose {
                println!("  🍑 F‘{}’ X«{}»", confusable_2, post_context);
            }
            post_context_map.entry(post_context.to_string()).or_default().push(confusable_2);
        }
        if let Some(pre_context) = line.strip_suffix(&space_c1) {
            if verbose {
                println!("  🍉 X«{}» F‘{}’", pre_context, confusable_1);
            }
            pre_context_map.entry(pre_context.to_string()).or_default().push(confusable_1);
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
            case_analysis.entry(lower).or_default().push(context.clone());
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
            case_analysis.entry(lower).or_default().push(context.clone());
        }
    }

    println!("\n🎯 Confusable contexts:");
    for (confusable, (pres, posts)) in &confusable_contexts {
        println!("  ‘{}’: pre={:?}, post={:?}", confusable, pres, posts);
    }

    if verbose {
        println!("\n🔤 Case sensitivity analysis:");
    }
    let mut case_sensitive_contexts: Vec<&str> = Vec::new();
    let mut case_insensitive_contexts: Vec<&str> = Vec::new();
    
    for (lower, variants) in &case_analysis {
        let unique_variants: Vec<_> = variants.iter().collect::<std::collections::HashSet<_>>().into_iter().cloned().collect();
        if unique_variants.len() == 1 {
            if verbose {
                println!("  CASE-SENSITIVE: ‘{}’ (only: {:?})", lower, unique_variants);
            }
            case_sensitive_contexts.push(lower);
        } else {
            if verbose {
                println!("  CASE-INSENSITIVE: ‘{}’ (variants: {:?})", lower, unique_variants);
            }
            case_insensitive_contexts.push(lower);
        }
    }
    
    println!("\n📋 Summary:");
    println!("  Case-sensitive contexts: {}", case_sensitive_contexts.len());
    println!("  Case-insensitive contexts: {}", case_insensitive_contexts.len());

    Ok(())
}
