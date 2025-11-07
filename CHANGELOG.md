# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.10.0] - 2025-01-XX

### Added

#### 🎉 Automatic Message Registration (Major Feature)

- **New `EventworkMessage` trait** - Automatically implemented for all `Serialize + Deserialize + Send + Sync + 'static` types
- **New `register_network_message<T>()`** - Register any serializable type as a network message without implementing `NetworkMessage`
- **Automatic type name generation** - Uses `std::any::type_name()` with caching for performance
- **New `send<T>()`** - Simplified send method that works with any `EventworkMessage` type
- **Updated `broadcast<T>()`** - Now works with any `EventworkMessage` type
- **External crate support** - Use types from any crate as network messages without wrapper types
- **Helper methods** - Added `is_message_registered()` and `registered_message_names()` for testing/debugging

#### Examples & Documentation

- **New `automatic_messages` example** - Complete working example demonstrating the new API
- **Comprehensive README** - Added examples/README.md with usage patterns and migration guide
- **Updated main README** - Showcases new API and includes migration guide

#### Tests

- **12 new integration tests** - Comprehensive test coverage for automatic message registration
- **Unit tests** - Tests for type name generation and caching behavior

### Changed

- **Improved ergonomics** - Reduced boilerplate by eliminating need for `NetworkMessage` trait in most cases
- **Better error messages** - More descriptive error messages for registration failures

### Deprecated

- `listen_for_message<T>()` - Use `register_network_message<T>()` instead (still fully functional)
- `send_message()` - Use `send()` instead (still fully functional)

**Note:** Deprecated methods are still fully supported and will continue to work. They are useful when you need explicit control over message names (e.g., for versioning).

### Fixed

- **Binary codec decode** - Fixed decode implementation in `eventwork_common` that was not properly handling length prefix

### Migration Guide

#### For New Code

Use the new automatic API:

```rust
// Before (0.9)
impl NetworkMessage for MyMessage {
    const NAME: &'static str = "my:Message";
}
app.listen_for_message::<MyMessage, TcpProvider>();
net.send_message(conn_id, msg)?;

// After (0.10)
#[derive(Serialize, Deserialize, Clone)]
struct MyMessage { /* fields */ }

app.register_network_message::<MyMessage, TcpProvider>();
net.send(conn_id, msg)?;
```

#### For Existing Code

No changes required! The old API continues to work:

```rust
// This still works exactly as before
app.listen_for_message::<MyMessage, TcpProvider>();
net.send_message(conn_id, msg)?;
```

To remove deprecation warnings, update to the new API:

```rust
app.register_network_message::<MyMessage, TcpProvider>();
net.send(conn_id, msg)?;
```

#### When to Use Each API

- **Use `register_network_message()`** - For most use cases, especially with external types
- **Use `listen_for_message()`** - When you need explicit message names (e.g., `"auth:v2:Login"` for versioning)

### Performance

- **Zero runtime overhead** - Type names are cached using `OnceCell`, computed once and reused
- **Same performance as const str** - After first access, no performance difference from explicit names

### Breaking Changes

None! This release is fully backward compatible.

### Version Updates

- `eventwork`: 0.9.11 → 0.10.0
- `eventwork_common`: 0.2.8 → 0.3.0
- `eventwork_websockets`: 0.2.1 → 0.3.0

---

## [0.9.11] - Previous Release

See git history for previous changes.

