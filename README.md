# cuda-tuple-space

Linda tuple space — generative communication, pattern matching, out/rd/in operations (Rust)

Part of the Cocapn communications layer — inter-agent messaging and data exchange.

## What It Does

### Key Types

- `Tuple` — core data structure
- `TupleSpace` — core data structure

## Quick Start

```bash
# Clone
git clone https://github.com/Lucineer/cuda-tuple-space.git
cd cuda-tuple-space

# Build
cargo build

# Run tests
cargo test
```

## Usage

```rust
use cuda_tuple_space::*;

// See src/lib.rs for full API
// 11 unit tests included
```

### Available Implementations

- `Tuple` — see source for methods
- `TupleSpace` — see source for methods

## Testing

```bash
cargo test
```

11 unit tests covering core functionality.

## Architecture

This crate is part of the **Cocapn Fleet** — a git-native multi-agent ecosystem.

- **Category**: comms
- **Language**: Rust
- **Dependencies**: See `Cargo.toml`
- **Status**: Active development

## Related Crates

- [cuda-communication](https://github.com/Lucineer/cuda-communication)
- [cuda-vessel-bridge](https://github.com/Lucineer/cuda-vessel-bridge)

## Fleet Position

```
Casey (Captain)
├── JetsonClaw1 (Lucineer realm — hardware, low-level systems, fleet infrastructure)
├── Oracle1 (SuperInstance — lighthouse, architecture, consensus)
└── Babel (SuperInstance — multilingual scout)
```

## Contributing

This is a fleet vessel component. Fork it, improve it, push a bottle to `message-in-a-bottle/for-jetsonclaw1/`.

## License

MIT

---

*Built by JetsonClaw1 — part of the Cocapn fleet*
*See [cocapn-fleet-readme](https://github.com/Lucineer/cocapn-fleet-readme) for the full fleet roadmap*
