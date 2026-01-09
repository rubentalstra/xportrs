# XPT Version 8

This module is reserved for future XPT v8 implementation.

## Key Differences from V5

XPT v8 (also known as extended transport format) differs from v5 in several ways:

### Variable Names
- V5: Maximum 8 bytes
- V8: Maximum 32 bytes

### Labels
- V5: Maximum 40 bytes
- V8: Maximum 256 bytes

### Header Structure
- Different magic strings and header record layouts
- Extended NAMESTR format to accommodate longer names

### Character Set
- V8 typically uses UTF-8 encoding
- V5 traditionally uses ASCII/Latin-1

## Implementation Status

**Not Implemented**

The API is ready for V8 selection (`XptVersion::V8`), but attempting to
read or write V8 files will return `XportrsError::UnsupportedVersion`.

## When to Use V8

V8 is useful when:
- Variable names exceed 8 characters
- Labels exceed 40 characters
- UTF-8 support is required

However, note that many regulatory submissions still require V5 format
for compatibility with older SAS versions.

## Future Implementation

When implementing V8 support, the following modules will be needed:

```
v8/
  mod.rs
  constants.rs    # V8 magic strings and markers
  namestr.rs      # Extended NAMESTR format
  read/
    mod.rs
    reader.rs
    parse.rs
  write/
    mod.rs
    writer.rs
```

## References

- SAS documentation on extended transport format
- CDISC guidance on XPT versions
