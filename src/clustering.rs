use crate::vprintln;

/// Extracts the statistically significant high-frequency core phrases
/// using a rolling baseline divergence threshold.
pub fn extract_core_cluster<'a>(verbose: bool, ranked_cores: &[(&'a str, u32)]) -> Vec<&'a str> {
    vprintln!(verbose, "\n📊 Isolating high-frequency variant cluster...");

    if ranked_cores.is_empty() {
        return Vec::new();
    }

    // A 40% drop-off from the rolling average signals the start of the long-tail
    let divergence_cutoff = 0.40;
    let mut cluster_size = 1;
    let mut cumulative_score = ranked_cores[0].1 as f64;

    for i in 1..ranked_cores.len() {
        let current_score = ranked_cores[i].1 as f64;
        let rolling_baseline = cumulative_score / (cluster_size as f64);

        let relative_step_down = (rolling_baseline - current_score) / rolling_baseline;

        if relative_step_down > divergence_cutoff {
            break;
        }

        cumulative_score += current_score;
        cluster_size += 1;
    }

    if verbose {
        println!("🤖 Cluster Identification Results:");
        println!("  📦 Core Cluster Size: {}", cluster_size);

        for (i, &(phrase, score)) in ranked_cores.iter().enumerate() {
            if i < cluster_size {
                println!("    👉 Core: '{}' (Score: {})", phrase, score);
            } else if i < cluster_size + 3 {
                println!("  📉 [Long-Tail] Phrase: '{}' (Score: {})", phrase, score);
            }
        }

        if ranked_cores.len() > cluster_size + 3 {
            println!(
                "    ... and {} more long-tail items.",
                ranked_cores.len() - cluster_size - 3
            );
        }
        println!("---------------------------------------------------\n");
    }

    // Return the clean slices back to main
    ranked_cores
        .iter()
        .take(cluster_size)
        .map(|&(phrase, _)| phrase)
        .collect()
}
