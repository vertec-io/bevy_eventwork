---
title: Release Notes
---
# Release Notes

Release notes and changelogs for the bevy_eventwork ecosystem.

## Releases

| Version | Date | Bevy | Highlights |
|---------|------|------|------------|
| [v0.10.0](./RELEASE_NOTES_v0.10.0.md) | 2025-11 | 0.17 | Automatic message registration |

## Version History

### 1.1.x (Current)

- Bevy 0.17 support
- Rust 2024 edition
- eventwork_sync and eventwork_client crates
- ExclusiveControlPlugin for control transfer patterns
- DevTools for debugging

### 0.10.x

- **Automatic message registration** - No more `NetworkMessage` trait boilerplate
- New `register_network_message()` API
- Simplified `send()` and `broadcast()` methods
- Full backward compatibility with existing code

### 0.9.x

- Bevy 0.16 support
- WebSocket transport provider
- Improved documentation

### 0.8.x

- Bevy 0.12 support
- Request/response patterns
- Connection pooling

## Upgrade Guides

For detailed upgrade instructions, see the [Migration](../migration/) section:

- [Bevy 0.17 Migration](../migration/MIGRATION_0.17.md)

## Versioning Policy

All eventwork crates are versioned together:

| Crate | Version |
|-------|---------|
| eventwork | 1.1.1 |
| eventwork_common | 1.1.1 |
| eventwork_websockets | 1.1.1 |
| eventwork_macros | 1.1.1 |
| eventwork_memory | 1.1.1 |
| eventwork_sync | 1.1.1 |
| eventwork_client | 1.1.1 |

**Always use matching versions** of all eventwork crates to avoid compatibility issues.

## Related Documentation

- [Migration Guides](../migration/) - Version upgrade instructions
- [Getting Started](../getting-started/) - Quick start guides

