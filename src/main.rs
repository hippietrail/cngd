use std::{
    collections::HashMap,
    io::{self, BufRead},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stored_lines: Vec<String> = Vec::new();
    let mut potential_cores: HashMap<String, u32> =
        HashMap::new();
    for line_result in io::stdin().lock().lines() {
        let line = line_result?;
        let line = line.trim();

        stored_lines.push(line.to_string());

        // Fail immediately if the line is empty
        if line.is_empty() {
            return Err("Program failed: Empty line encountered".into());
        }

        // Split from the left to isolate the first word
        let (first, maybe_core_end) = line
            .split_once(' ')
            .ok_or("Program failed: Missing space separator")?;

        // Split from the right to isolate the core and last word
        let (maybe_core_start, last) = line
            .rsplit_once(' ')
            .ok_or("Program failed: Missing space separator")?;

        let (maybe_core_end, maybe_core_start) =
            (maybe_core_end.to_string(), maybe_core_start.to_string());

        // use ansi colours for the string parts and ansi bold+italic for "OR"
        println!(
            "\x1b[34m{}\x1b[0m + \x1b[35m{}\x1b[0m \x1b[1;3mOR\x1b[0m \x1b[36m{}\x1b[0m + \x1b[37m{}\x1b[0m",
            first, maybe_core_end, maybe_core_start, last
        );

        // If we don't have any potential cores yet, add both of these
        if potential_cores.is_empty() {
            eprintln!(
                "Adding potential cores: {} and {}",
                maybe_core_end, maybe_core_start
            );
            potential_cores.insert(maybe_core_end.clone(), 1);
            potential_cores.insert(maybe_core_start.clone(), 1);
            // we may have a problem if they were both the same - we can compare them or see how many items we have in the vec
            if potential_cores.len() != 2 {
                return Err("Program failed: Potential cores are the same".into());
            }
        } else {
            *potential_cores.entry(maybe_core_end.clone()).or_insert(0) += 1;
            *potential_cores.entry(maybe_core_start.clone()).or_insert(0) += 1;
        }
    }

    // Print the potential cores and their counts in descending order
    let mut potential_cores_vec: Vec<(String, u32)> = potential_cores.into_iter().collect();
    potential_cores_vec.sort_by(|a, b| b.1.cmp(&a.1));
    for (core, count) in potential_cores_vec.iter().take(6) {
        println!("{}: {}", core, count);
    }

    // assume the top 2 are the confusable terms
    let confusable_1 = &potential_cores_vec[0].0;
    let confusable_2 = &potential_cores_vec[1].0;

    // 2. Identify and collect ambiguous context words
    // let's make something to map between an context and a set of confusables
    let mut pre_context_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut post_context_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for line in &stored_lines {
        if let Some(post_context) = line.strip_prefix(&format!("{} ", confusable_1)) {
            println!("  🍇 F‘{}’ X«{}»", confusable_1, post_context);
            post_context_map.entry(post_context.to_string()).or_default().push(confusable_1.clone());
        }
        if let Some(pre_context) = line.strip_suffix(&format!(" {}", confusable_2)) {
            println!("  🍐 X«{}» F‘{}’", pre_context, confusable_2);
            pre_context_map.entry(pre_context.to_string()).or_default().push(confusable_2.clone());
        }
        if let Some(post_context) = line.strip_prefix(&format!("{} ", confusable_2)) {
            println!("  🍑 F‘{}’ X«{}»", confusable_2, post_context);
            post_context_map.entry(post_context.to_string()).or_default().push(confusable_2.clone());
        }
        if let Some(pre_context) = line.strip_suffix(&format!(" {}", confusable_1)) {
            println!("  🍉 X«{}» F‘{}’", pre_context, confusable_1);
            pre_context_map.entry(pre_context.to_string()).or_default().push(confusable_1.clone());
        }
    }

    let mut confusable_1_to_pres_and_posts: HashMap<String, (Vec<String>, Vec<String>)> = std::collections::HashMap::new();
    let mut confusable_2_to_pres_and_posts: HashMap<String, (Vec<String>, Vec<String>)> = std::collections::HashMap::new();


    println!("Pre-context map:");
    for (context, confusables) in pre_context_map {
        if confusables.len() == 1 {
            println!("  ‘{}’: {:?}", context, confusables);
            // is this confusable1 or confusable2?
            if confusables[0] == *confusable_1 {
                confusable_1_to_pres_and_posts.entry(confusables[0].clone()).or_default().0.push(context.clone());
            } else {
                confusable_2_to_pres_and_posts.entry(confusables[0].clone()).or_default().0.push(context.clone());
            }
        }
    }

    println!("Post-context map:");
    for (context, confusables) in post_context_map {
        if confusables.len() == 1 {
            println!("  ‘{}’: {:?}", context, confusables);
            // is this confusable1 or confusable2?
            if confusables[0] == *confusable_1 {
                confusable_1_to_pres_and_posts.entry(confusables[0].clone()).or_default().1.push(context.clone());
            } else {
                confusable_2_to_pres_and_posts.entry(confusables[0].clone()).or_default().1.push(context.clone());
            }
        }
    }

    println!("Confusable 1 to pre/post contexts:");
    for (confusable, contexts) in confusable_1_to_pres_and_posts {
        println!("  ‘{}’: {:?}", confusable, contexts);
    }

    println!("Confusable 2 to pre/post contexts:");
    for (confusable, contexts) in confusable_2_to_pres_and_posts {
        println!("  ‘{}’: {:?}", confusable, contexts);
    }

    Ok(())
}
