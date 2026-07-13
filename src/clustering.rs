/// Extracts the statistically significant high-frequency core phrases 
/// using a rolling baseline divergence threshold.
pub fn extract_core_cluster<'a>(verbose: bool, ranked_cores: &[(&'a str, u32)]) -> Vec<&'a str> {
    if verbose {
        println!("\n📊 Isolating high-frequency variant cluster...");
    }

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
            println!("    ... and {} more long-tail items.", ranked_cores.len() - cluster_size - 3);
        }
        println!("---------------------------------------------------\n");
    }

    // Return the clean slices back to main
    ranked_cores.iter()
        .take(cluster_size)
        .map(|&(phrase, _)| phrase)
        .collect()
}
// pub fn analyze_arbitrary_confusables(sorted_cores: &[(&str, u32)]) {
//     println!("\n🚀 Running Shadow Analysis on Frequency Scores...");

//     if sorted_cores.len() < 2 {
//         println!("  ⚠️ Not enough data points to analyze.");
//         return;
//     }

//     // 1. Define a drop-off threshold.
//     // If the next word's frequency drops by more than 40% compared to the
//     // average of the top group, we consider the high-frequency cluster finished.
//     let drop_threshold = 0.40;

//     let mut confusable_count = 1; // The absolute top word is always included
//     let mut running_sum = sorted_cores[0].1 as f64;

//     for i in 1..sorted_cores.len() {
//         let current_score = sorted_cores[i].1 as f64;
//         let running_avg = running_sum / (confusable_count as f64);

//         // Calculate how much the score drops relative to our high-frequency average
//         let drop = (running_avg - current_score) / running_avg;

//         // If a significant disjoint gap occurs, stop including words
//         if drop > drop_threshold {
//             break;
//         }

//         running_sum += current_score;
//         confusable_count += 1;
//     }

//     // 2. Print recommendations based on the natural gap boundary
//     println!("🤖 Score-Gap Cluster Recommendations:");
//     println!(
//         "  📦 Detected Confusable Group (Size {}):",
//         confusable_count
//     );

//     for (i, &(word, score)) in sorted_cores.iter().enumerate() {
//         if i < confusable_count {
//             println!("    👉 Word: '{}' (Score: {})", word, score);
//         } else if i < confusable_count + 3 {
//             // Show just a few noise items for context
//             println!(
//                 "  🧹 [Noise/Outlier]   Word: '{}' (Score: {})",
//                 word, score
//             );
//         }
//     }

//     if sorted_cores.len() > confusable_count + 3 {
//         println!(
//             "    ... and {} more noise items.",
//             sorted_cores.len() - confusable_count - 3
//         );
//     }
//     println!("---------------------------------------------------\n");
// }
