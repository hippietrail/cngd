"cngd" stands for "contextual n-gram differencer" that Google Search's AI spontaneously called it when I proposed the idea.

The idea is that you can use Google NGrams with two confusable terms in combination with the `*` wildcard before and after each one to see the most common previous and next words when each term is used.

You can copy and paste the column of variants from the right side of the graph and save into a text file.

## Usage

```bash
# Basic usage - auto-detect the two most frequent confusable terms
cngd < variants.txt

# Specify confusable terms manually
cngd < variants.txt term1 term2

# Enable automatic clustering for multiple variants
cngd --auto-cluster < variants.txt

# Verbose output to see intermediate analysis steps
cngd -v < variants.txt
cngd --verbose < variants.txt

# Combine options
cngd --auto-cluster -v < variants.txt
```

## Command Line Options

- `-v`, `--verbose` - Output intermediate steps of the analysis
- `--auto-cluster` - Automatically detect clusters of confusable terms using frequency-based analysis
- `[terms...]` - Manually specify which confusable terms to analyze (space-separated)

## How It Works

1. **Input**: Reads phrase variants from stdin (one per line)
2. **Core Detection**: Identifies potential confusable terms by frequency (or uses manual/auto-cluster selection)
3. **Context Extraction**: For each confusable term, extracts:
   - Pre-contexts: words that appear before the term
   - Post-contexts: words that appear after the term
4. **Filtering**: Eliminates contexts shared between confusables to find discriminative contexts
5. **Case Analysis**: Classifies contexts as case-sensitive or case-insensitive
6. **POS Analysis**: Uses Harper's dictionary to identify part-of-speech patterns for contexts
7. **Output**: Color-coded results showing which contexts can distinguish between confusable terms

## Output

The tool outputs:
- **Confusable contexts**: Which previous/next words uniquely identify each confusable term
- **POS associations**: Part-of-speech patterns for the discriminative contexts
- **Case sensitivity**: Whether contexts are case-sensitive or case-insensitive

This information can be used to create grammar checker rules that help determine when a confusable word is used correctly or mistakenly.

----

I initially coded this by hand, but with help from the AI assistant built into Devin, the code editor formerly known as Windsurf, and from Google Search's AI.

Once I had it working as I wanted, I got Devin to refactor it to be more idiomatic Rust and then add some trivial features.

I also got Devin to write the "future direction" markdown file.
