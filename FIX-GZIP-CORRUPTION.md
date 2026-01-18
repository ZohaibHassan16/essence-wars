# Gzip Corruption Fix

## Problem
The 100k MCTS dataset (`mcts_100k_sims100_20260118_162457.jsonl.gz`) was corrupted - the gzip stream ended prematurely without proper finalization, making it impossible to decompress.

## Root Cause
In [generate_dataset.rs](src/bin/generate_dataset.rs), the `GzEncoder` was being dropped without explicitly calling `finish()` to finalize the gzip stream. When using gzip compression, you must:

1. Flush the `BufWriter`
2. Unwrap the `BufWriter` to get the `GzEncoder`
3. Call `encoder.finish()` to write the gzip end-of-stream marker

## Fix
Updated the writer thread code to properly finalize gzip streams:

```rust
// Before (BROKEN - missing finalization)
let encoder = GzEncoder::new(file, Compression::default());
let mut writer = BufWriter::new(encoder);
for record in rx {
    serde_json::to_writer(&mut writer, &record)?;
    writeln!(writer)?;
}
// writer/encoder dropped without finalizing stream

// After (FIXED - explicit finalization)
let encoder = GzEncoder::new(file, Compression::default());
let mut writer = BufWriter::new(encoder);
for record in rx {
    serde_json::to_writer(&mut writer, &record)?;
    writeln!(writer)?;
}
writer.flush()?;  // Flush buffer
let encoder = writer.into_inner()?;  // Unwrap encoder
encoder.finish()?;  // Write end-of-stream marker
```

## Testing
Verified locally with 100-game dataset:
```bash
./target/release/generate_dataset --games 100 --sims 50 --output test.jsonl.gz
gzip -t test.jsonl.gz  # ✓ Valid
```

## Next Steps
1. **Regenerate the 100k dataset on Modal** with the fixed code
2. Re-run analysis to verify data quality
3. Publish to Hugging Face once validated

## Analysis Script Updates
Also updated `scripts/analyze_dataset.py` to:
- Handle corrupted gzip files gracefully (report partial data)
- Match new dataset format (`moves` array instead of separate `states`/`actions`)
- Properly validate dataset structure for publication readiness
