# Future Directions for Confusable Word Analyzer

## Current Limitations

### Inflected Words
The current analyzer treats each distinct string as a separate candidate, which doesn't work well for morphological variants:

**Example: contend-vs-contest.txt**
- Contains 6 different forms: contended, contested, contends, contests, contending, contesting
- Current approach picks "contends with" and "contests with" as top 2
- But all 6 forms are actually variants of the same 2 base words

**Needed improvements:**
- Lemmatization or stemming to group morphological variants
- Could use libraries like `lemmatizer` or integrate with NLP tools
- Consider part-of-speech tagging to handle different word classes

### Variable Set Sizes
The current approach assumes exactly 2 confusable terms, but real data may have more:

**Examples:**
- `contend-vs-contest.txt`: 6 variants (contended, contested, contends, contests, contending, contesting)
- `wide-ranging-vs-wide-sweeping.txt`: 4 variants (wide sweeping, wide - sweeping, wide ranging, wide - ranging)

**Needed improvements:**
- Clustering algorithms to automatically determine the number of confusable groups
- Similarity metrics to group related terms
- Configurable threshold for grouping

### Hyphenation and Punctuation
**Example: wide-ranging-vs-wide-sweeping.txt**
- "wide sweeping" vs "wide - sweeping" treated as separate
- Hyphenation patterns vary in corpora

**Needed improvements:**
- Normalization of punctuation/spacing
- Configurable tokenization rules
- Handle en-dashes, em-dashes, and other punctuation

### Case Sensitivity Analysis
**Example: trouble-vs-troubles.txt**
- 22 case-sensitive contexts (only appear in one case form)
- 1 case-insensitive context: "the" appears as both "the" and "The"
- This suggests most context words are consistently cased in the corpus

**Example: border-vs-boarder.txt**
- 23 case-sensitive contexts
- 0 case-insensitive contexts
- All context words appear in consistent case

**Implementation for Harper:**
- Track case variations for each context word
- Classify contexts as:
  - **Case-sensitive**: Only one case form observed (e.g., "Mexican", "posterior")
  - **Case-insensitive**: Multiple case forms observed (e.g., "the"/"The")
- Generate separate rule recommendations:
  - Use case-sensitive matching for words that only appear in one form
  - Use case-insensitive matching for words that appear in multiple forms
- This data-driven approach avoids over-generalization while handling natural case variation

### Multi-word Phrases
**Example: wide-ranging vs wide-sweeping**
- Both are 2-word phrases
- Current splitting logic assumes single-word cores

**Needed improvements:**
- Support for multi-word confusable terms
- Configurable phrase length detection
- N-gram analysis for phrase boundaries

## Proposed Architecture

### 1. Preprocessing Pipeline
```
Raw text → Normalization → Tokenization → Lemmatization → Feature extraction
```

### 2. Clustering Stage
- Use similarity metrics (Levenshtein, Jaccard, embedding-based)
- Hierarchical clustering to discover groups
- Automatic determination of cluster count

### 3. Context Analysis
- Expand context window beyond immediate neighbors
- Weight contexts by frequency and discriminative power
- Statistical significance testing

### 4. Output Generation
- Grouped results with confidence scores
- Contextual examples for each group
- Statistical summaries

## Data Quality Considerations

### Input Format
- Current format: one phrase per line
- Could support:
  - Sentence-level input with automatic extraction
  - Frequency counts alongside phrases
  - Metadata (source, date, domain)

### Validation
- Manual verification of confusable pairs
- Cross-validation with dictionaries
- False positive/negative analysis
