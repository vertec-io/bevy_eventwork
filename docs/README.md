# bevy_eventwork Documentation

This directory contains comprehensive documentation for bevy_eventwork and general Bevy upgrade resources.

## 📚 Documentation Index

### Architecture & Design

| Document | Description | Audience |
|----------|-------------|----------|
| [EVENTWORK_SYNC_ARCHITECTURE.md](./EVENTWORK_SYNC_ARCHITECTURE.md) | Complete architecture guide for eventwork_sync and eventwork_client, including the Meteorite pattern, wire protocol, and subscription flow | Developers using eventwork_sync |
| [EVENTWORK_SYNC_PERFORMANCE.md](./EVENTWORK_SYNC_PERFORMANCE.md) | Performance analysis, best practices, and optimization guide for eventwork_sync applications | Developers optimizing sync performance |

### For bevy_eventwork Users

| Document | Description | Audience |
|----------|-------------|----------|
| [MIGRATION_0.17.md](./MIGRATION_0.17.md) | Complete migration guide for upgrading from Bevy 0.16 to 0.17 with bevy_eventwork | bevy_eventwork users |
| [RUST_NIGHTLY_REQUIREMENT.md](./RUST_NIGHTLY_REQUIREMENT.md) | Detailed explanation of why nightly Rust is required and when to switch back to stable | All Bevy 0.17 users |

### For AI-Assisted Bevy Upgrades

| Document | Description | Audience |
|----------|-------------|----------|
| [upgrade-bevy-017.md](./upgrade-bevy-017.md) | **Complete upgrade execution instructions** - Single command for AI agents | **AI agents** |
| [BEVY_0.17_UPGRADE_PRE_PLANNING.md](./BEVY_0.17_UPGRADE_PRE_PLANNING.md) | Research & planning instructions (referenced by upgrade-bevy-017.md) | AI agents |
| [HOW_TO_USE_UPGRADE_TEMPLATE.md](./HOW_TO_USE_UPGRADE_TEMPLATE.md) | Guide for developers using AI agents for upgrades | Developers |

### For General Bevy Projects

| Document | Description | Audience |
|----------|-------------|----------|
| [BEVY_0.17_MIGRATION_GUIDE.md](./BEVY_0.17_MIGRATION_GUIDE.md) | Standardized Bevy 0.17 migration guide for any Bevy project | Any Bevy developer |

## 🚀 Quick Start

### Upgrading bevy_eventwork Projects

If you're using bevy_eventwork and upgrading to Bevy 0.17:

1. Read [MIGRATION_0.17.md](./MIGRATION_0.17.md)
2. Follow the step-by-step migration instructions
3. Refer to [RUST_NIGHTLY_REQUIREMENT.md](./RUST_NIGHTLY_REQUIREMENT.md) for Rust setup

### Upgrading Other Bevy Projects

If you're upgrading a different Bevy project to 0.17:

1. Read [BEVY_0.17_MIGRATION_GUIDE.md](./BEVY_0.17_MIGRATION_GUIDE.md) for general guidance
2. Use the quick reference table for find/replace patterns
3. Follow the verification checklist

### Using AI Agents for Upgrades

**Simplest approach** - Single command:

1. Copy [upgrade-bevy-017.md](./upgrade-bevy-017.md) to your project
2. Tell AI agent: `"Read docs/upgrade-bevy-017.md and execute it"`
3. Done! The agent will handle research, implementation, and documentation

**For more control** - Two-phase approach:

1. Read [HOW_TO_USE_UPGRADE_TEMPLATE.md](./HOW_TO_USE_UPGRADE_TEMPLATE.md)
2. Copy [BEVY_0.17_UPGRADE_PRE_PLANNING.md](./BEVY_0.17_UPGRADE_PRE_PLANNING.md) to your project
3. Tell AI agent to execute research phase first
4. Review research, then tell agent to execute implementation

## 📖 Document Summaries

### upgrade-bevy-017.md ⭐

**Purpose**: Complete execution instructions for AI agents to perform Bevy 0.17 upgrades

**Contents**:
- Complete mission and success criteria
- Three-phase process (Research → Implementation → Documentation)
- Specific implementation order
- Design philosophy (quality > backward compatibility)
- Comprehensive verification steps
- Documentation requirements

**When to use**: You want an AI agent to handle the entire upgrade process

**Usage**: Copy to your project and tell AI: `"Read docs/upgrade-bevy-017.md and execute it"`

---

### BEVY_0.17_UPGRADE_PRE_PLANNING.md

**Purpose**: Research and planning instructions for AI agents (referenced by upgrade-bevy-017.md)

**Contents**:
- Structured research requirements
- Step-by-step analysis guidelines
- Expected deliverables specification
- Success criteria
- Direct instructions for discovering project information

**When to use**: You want more control and want to review research before implementation

---

### HOW_TO_USE_UPGRADE_TEMPLATE.md

**Purpose**: Guide for developers using AI agents for Bevy upgrades

**Contents**:
- Two-phase workflow explanation
- Example workflows with timelines
- Best practices and troubleshooting
- Success metrics

**When to use**: You want to understand the AI-assisted upgrade process

---

### MIGRATION_0.17.md

**Purpose**: Help bevy_eventwork users migrate from Bevy 0.16 to 0.17

**Contents**:
- Overview of Bevy 0.17 changes
- Breaking changes specific to bevy_eventwork
- Step-by-step migration instructions
- Complete code examples (server and client)
- Troubleshooting common issues

**When to use**: You're upgrading a project that uses bevy_eventwork

---

### BEVY_0.17_MIGRATION_GUIDE.md

**Purpose**: Provide a standardized migration guide for any Bevy 0.17 upgrade

**Contents**:
- Complete overview of Bevy 0.17 breaking changes
- Event → Message migration patterns
- Common code patterns with before/after examples
- Troubleshooting section
- Quick reference table for find/replace
- Verification checklist

**When to use**: You're upgrading any Bevy project (not just bevy_eventwork)

---

### RUST_NIGHTLY_REQUIREMENT.md

**Purpose**: Explain why nightly Rust is required for Bevy 0.17 and when to switch back

**Contents**:
- Technical background on Rust 1.88.0 requirement
- Expected timeline for stable release (Q1 2026)
- Setup instructions for nightly Rust
- Safety considerations
- CI/CD configuration examples
- Comprehensive FAQ

**When to use**: You need to understand or explain the nightly Rust requirement

---

## 🎯 Use Case Guide

### "I want an AI agent to do the entire upgrade"

→ Copy [upgrade-bevy-017.md](./upgrade-bevy-017.md) and tell AI: `"Read docs/upgrade-bevy-017.md and execute it"`

### "I want to review research before implementation"

→ Start with [HOW_TO_USE_UPGRADE_TEMPLATE.md](./HOW_TO_USE_UPGRADE_TEMPLATE.md), use two-phase approach

### "I'm upgrading my bevy_eventwork project manually"

→ Start with [MIGRATION_0.17.md](./MIGRATION_0.17.md)

### "I'm upgrading a different Bevy project manually"

→ Start with [BEVY_0.17_MIGRATION_GUIDE.md](./BEVY_0.17_MIGRATION_GUIDE.md)

### "Why do I need nightly Rust?"

→ Read [RUST_NIGHTLY_REQUIREMENT.md](./RUST_NIGHTLY_REQUIREMENT.md)

### "I want to reuse this process for other projects"

→ Copy [upgrade-bevy-017.md](./upgrade-bevy-017.md), [BEVY_0.17_UPGRADE_PRE_PLANNING.md](./BEVY_0.17_UPGRADE_PRE_PLANNING.md), and [BEVY_0.17_MIGRATION_GUIDE.md](./BEVY_0.17_MIGRATION_GUIDE.md)

## 🔧 Key Concepts

### Event → Message Migration

The most significant change in Bevy 0.17 is the event system split:

| Bevy 0.16 | Bevy 0.17 | Notes |
|-----------|-----------|-------|
| `Event` | `Message` | For buffered events (most common) |
| `EventReader<T>` | `MessageReader<T>` | Reading buffered events |
| `EventWriter<T>` | `MessageWriter<T>` | Writing buffered events |
| `.send()` | `.write()` | Method name change |
| `add_event::<T>()` | `add_message::<T>()` | App registration |

### Rust Nightly Requirement

- **Required**: Rust 1.88.0 (not yet in stable)
- **Current solution**: Use nightly Rust
- **Setup**: Create `rust-toolchain.toml` with `channel = "nightly"`
- **When to switch back**: Q1 2026 (when Rust 1.88.0 is stable)

### AI-Assisted Upgrade Process

**Single-command approach** (Simplest):
- Copy `upgrade-bevy-017.md` to your project
- Tell AI: `"Read docs/upgrade-bevy-017.md and execute it"`
- Agent handles everything: research → implementation → documentation

**Two-phase approach** (More control):
1. **Research** (AI Agent) - Agent researches and creates upgrade plan
2. **Implementation** (AI Agent) - After your review, agent executes the upgrade

This approach reduces risk and improves success rate.

## 📝 Contributing

If you find issues or have suggestions for these documents:

1. Open an issue on [GitHub](https://github.com/jamescarterbell/bevy_eventwork/issues)
2. Submit a pull request with improvements
3. Share your upgrade experience to help others

## 🔗 External Resources

- [Official Bevy 0.17 Migration Guide](https://bevyengine.org/learn/migration-guides/0-16-to-0-17/)
- [Bevy 0.17 Release Notes](https://bevyengine.org/news/bevy-0-17/)
- [Bevy Book](https://bevyengine.org/learn/book/introduction/)
- [Bevy Discord](https://discord.gg/bevy)
- [Rust Nightly Documentation](https://doc.rust-lang.org/nightly/)

## 📄 License

All documentation in this directory is provided under the same license as bevy_eventwork (MIT OR Apache-2.0).

---

**Last Updated**: November 9, 2025  
**Bevy Version**: 0.17.2  
**bevy_eventwork Version**: 1.1.1

