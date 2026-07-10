"cngd" stands for "contextual n-gram differencer" that Google Search's AI spontaneously called it when I proposed the idea.

The idea is that you can use Google NGrams with two confuable terms in combination with the `*` wildcard before and after each one to see the most common previous and next words when each term is used.

You can copy and paste the column of variants from the right side of the graph and save into a text file.

Then run cngd redirecting input from that file:

```bash
cngd < variants.txt
```

The tool will try to figure out what the two confusable terms are. Yes it's limited to two confusables at this time.

Then it will look at the previous-word context and next-word context of each and eliminate any contexts that are shared between both confusables.

At the end it will output which previous and which next context words can be used in a grammar checker rule as heuristics to help decide when a confusable word is used correctly or mistakenly instead of the other one.

With the `-v` or `--verbose` flags, it will output the intermediate steps of the analysis.

----

I initially coded this by hand, but with help from the AI assistant built into Devin, the code editor formerly known as Windsurf, and from Google Search's AI.

Once I had it working as I wanted, I got Devin to refactor it to be more idiomatic Rust and then add some trivial features.

I also got Devin to write the "future direction" markdown file.
