# Binary Export Usage Example

This example demonstrates how to use memscope-rs with binary export functionality, which provides efficient storage and conversion capabilities for memory analysis data.

## Running the Example

```bash
# Run the binary export example
cargo run --example basic_usage_binary
```

## What the Example Does

The `basic_usage_binary.rs` example is identical to `basic_usage.rs` in terms of memory tracking, but instead of exporting to JSON and SVG formats, it demonstrates the new binary export capabilities:

1. **Memory Tracking**: Tracks the same variables as the basic example
2. **Binary Export**: Exports memory data to binary format with different compression options
3. **Multiple Export Modes**: Demonstrates three different binary export configurations:
   - **Balanced** (default): Good compression with reasonable speed
   - **Fast**: No compression for maximum speed
   - **Compact**: Maximum compression for smallest file size

## Output Files

The example creates three binary files in `MemoryAnalysis/basic_usage/`:

- `basic_usage_snapshot.ms` - Balanced compression (recommended)
- `basic_usage_fast.ms` - No compression (fastest export)
- `basic_usage_compact.ms` - Maximum compression (smallest size)

## Converting Binary Files

After running the example, you can convert the binary files to other formats using the `memscope export` command:

### Convert to JSON
```bash
# Standard conversion
memscope export -i MemoryAnalysis/basic_usage/basic_usage_snapshot.ms -f json -o basic_usage.json

# Streaming conversion (for large files)
memscope export -i MemoryAnalysis/basic_usage/basic_usage_snapshot.ms -f json --streaming -o basic_usage_stream.json
```

### Convert to HTML
```bash
# Generate interactive HTML report
memscope export -i MemoryAnalysis/basic_usage/basic_usage_snapshot.ms -f html -o basic_usage.html
```

### Validate Binary File
```bash
# Validate file integrity
memscope export -i MemoryAnalysis/basic_usage/basic_usage_snapshot.ms --validate-only
```

## Binary Export Options

The example demonstrates three different `BinaryExportOptions`:

### Fast Export
- **Compression**: None
- **Speed**: Fastest
- **File Size**: Largest
- **Use Case**: Quick exports, temporary files

### Balanced Export (Default)
- **Compression**: zstd level 6
- **Speed**: Good
- **File Size**: Medium
- **Use Case**: General purpose, recommended for most users

### Compact Export
- **Compression**: zstd level 19
- **Speed**: Slower
- **File Size**: Smallest
- **Use Case**: Long-term storage, sharing files

## Advantages of Binary Format

1. **Efficiency**: Smaller file sizes with compression
2. **Speed**: Faster export and import compared to JSON
3. **Integrity**: Built-in validation and error recovery
4. **Flexibility**: Can be converted to any supported format later
5. **Metadata**: Includes export metadata and checksums

## Example Output

When you run the example, you'll see output like:

```
memscope-rs initialized. Tracking memory allocations...

Allocating and tracking variables...
Tracked 'numbers_vec'
Tracked 'text_string'
...

Memory Statistics:
  Active allocations: 7
  Active memory: 156 bytes
  Total allocations: 7
  Peak memory: 156 bytes

Exporting memory snapshot to binary format...
✅ Successfully exported binary to: MemoryAnalysis/basic_usage/basic_usage_snapshot.ms
📊 Export Statistics:
   - Export time: 2.1ms
   - File size: 245 bytes
   - Original size: 512 bytes
   - Compression ratio: 47.9%
   - Allocations exported: 7
   - Total memory tracked: 156 bytes

💡 Usage Instructions:
   Convert to JSON: memscope export -i MemoryAnalysis/basic_usage/basic_usage_snapshot.ms -f json -o basic_usage.json
   Convert to HTML: memscope export -i MemoryAnalysis/basic_usage/basic_usage_snapshot.ms -f html -o basic_usage.html
   Validate file:   memscope export -i MemoryAnalysis/basic_usage/basic_usage_snapshot.ms --validate-only
```

## Integration with Existing Workflow

The binary format is designed to integrate seamlessly with existing memscope-rs workflows:

1. **Export**: Use binary format for efficient storage
2. **Convert**: Convert to JSON/HTML when needed for analysis
3. **Share**: Binary files are compact and portable
4. **Validate**: Built-in validation ensures data integrity

This approach provides the best of both worlds: efficient storage with the flexibility to convert to human-readable formats when needed.