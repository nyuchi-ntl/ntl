# Nyuchi Transfer Layer (NTL)

**The Neural Transfer Layer for Modern Compute**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Spec Version](https://img.shields.io/badge/spec-v0.1.0--draft-orange.svg)](https://ntl.nyuchi.com/spec/overview)

---

NTL is an open source data transfer layer that replaces the request-response paradigm of traditional APIs with neural signal propagation. Built for the age of AI, Web3, and quantum computing.

## Why NTL?

Every major data transfer protocol in use today — HTTP, REST, GraphQL, gRPC — was designed for a world of clients and servers, requests and responses. That world is ending.

NTL introduces:

- **Signals** instead of requests — typed, weighted, cryptographically signed payloads
- **Synapses** instead of connections — persistent channels that strengthen with use
- **Activation thresholds** instead of rate limiting — intelligent, adaptive flow control
- **Emergent routing** instead of endpoint registries — the network self-organizes
- **Pluggable cryptography** — post-quantum ready, no hardcoded schemes

## Architecture

```
┌─────────────────────────────────┐
│         Applications            │
│      (Mukoko, dApps, AI)        │
├─────────────────────────────────┤
│     Nyuchi Transfer Layer       │  ← This project
│    (Neural Signal Transport)    │
├─────────────────────────────────┤
│          SiafuDB                │
│   (Swarm-based Graph Storage)   │
├─────────────────────────────────┤
│     Network / Hardware          │
│   (TCP/UDP/QUIC substrate)      │
└─────────────────────────────────┘
```

## Quick Start

```bash
# Install
cargo install ntl-cli

# Initialize a node
ntl init

# Start (development mode)
ntl start --dev

# Emit a signal
ntl emit --type data --payload '{"hello": "world"}'

# Listen for signals
ntl listen
```

## Documentation

Full documentation is available at [ntl.nyuchi.com](https://ntl.nyuchi.com).

- [Introduction](https://ntl.nyuchi.com/introduction)
- [Why NTL](https://ntl.nyuchi.com/why-ntl)
- [Architecture](https://ntl.nyuchi.com/architecture)
- [Core Concepts](https://ntl.nyuchi.com/concepts/signals)
- [Specification](https://ntl.nyuchi.com/spec/overview)
- [Quickstart Guide](https://ntl.nyuchi.com/guides/quickstart)

## Repository Structure

```
ntl/
├── spec/               # Protocol specification documents
├── runtime/            # Rust reference implementation
│   ├── ntl-core/       # Core library
│   ├── ntl-cli/        # CLI tooling
│   ├── ntl-node/       # Full node binary
│   └── ntl-edge/       # Edge node (lightweight)
├── adapters/           # Protocol adapters
│   ├── web2/           # HTTP, WebSocket, gRPC, GraphQL
│   ├── web3/           # EVM chains, DID, tokens
│   └── legacy/         # REST/SOAP wrapper
├── docs/               # Mintlify documentation source
├── rfcs/               # Request for Comments
├── examples/           # Example applications
└── benchmarks/         # Performance benchmarks
```

## Project Status

NTL is in **Phase 0: Foundation** — specification development and documentation. See the [roadmap](https://ntl.nyuchi.com/governance/roadmap) for details.

## Contributing

We welcome contributions from anyone, anywhere. See [CONTRIBUTING.md](CONTRIBUTING.md) and our [contribution guide](https://ntl.nyuchi.com/governance/contributing).

NTL is built on the Ubuntu philosophy — *"I am because we are."*

## Built by Nyuchi Africa

NTL is built by [Nyuchi Africa](https://nyuchi.com), a technology company in Zimbabwe building open source platforms and infrastructure for African markets and beyond.

| Entity | Role |
|---|---|
| [Nyuchi Africa](https://nyuchi.com) | Parent entity, stewardship |
| [Nyuchi Web Services](https://nws.nyuchi.com) | Engineering, reference implementation |
| [SiafuDB](https://siafu.nyuchi.com) | Companion storage layer |
| [Mukoko](https://mukoko.com) | Application platform |

## License

Apache 2.0 — see [LICENSE](LICENSE).
