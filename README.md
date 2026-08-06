# OpenMCPE

An independent re-implementation of Minecraft: Pocket Edition 0.6.1, written in Rust.

**Current status: early development.** Currently displays a triangle only. Not a playable game.

## Disclaimer

NOT AN OFFICIAL MINECRAFT PRODUCT.
NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.

Minecraft is a trademark of Mojang Studios. This project is not affiliated
with, endorsed by, or connected to Mojang or Microsoft in any way.

OpenMCPE is a personal study project. Its goal is to understand and re-create the
behaviour of the 0.6.1 alpha release of Minecraft: Pocket Edition as an exercise
in engine craft and software preservation. It is not a replacement for the
original game, and it is not a means of obtaining it.

## Assets

**This project contains no Minecraft assets — textures, sounds, fonts, or data files** — and none ever will be.

To build and run OpenMCPE, you must extract them from the original Minecraft you own, and place them under `assets/`.

## Building

Requires Rust 1.87 or newer (edition 2024).

```sh
cargo run
```

### Linux

X11 or Wayland development headers may be required.

## License

**None. All rights reserved.**

OpenMCPE is written from scratch, but its behaviour is derived from study of the
original game. The author does not claim standing to grant rights over that
lineage, and would rather offer no license than an unfounded one.

You are welcome to read this code for educational purposes. Please don't distribute or copy this project.
