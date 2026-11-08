# Chatter

A native Discord client for Windows and Linux, written in Rust with Slint. No
web browser inside, so it idles at a fraction of the memory Discord's own app
uses.

Chatter is a fork of [Yapfast](https://yapfast.com)
([github.com/yapfast](https://github.com/yapfast)). Almost everything that
makes it work, from voice and video to the Discord protocol itself, is their
work. Chatter adds its own look and a set of extras on top.

## What Chatter adds

**Look and feel**
- Discord-style settings layout and account switcher (flyout plus Manage
  Accounts)
- 12 themes
- Unread count badge on the taskbar icon

**Discord Tweaks**
- Message logger: deleted messages stay visible in red, and edits keep the
  old text underneath
- Show hidden channels
- Silent typing
- Spotify listening status, shown like Discord's own client shows it
- Streamer Mode, turned on automatically while OBS runs

Everything Yapfast already does is still here: voice, cameras, Go Live,
forums, polls, the GIF picker, server settings and more.

## Will Discord ban me?

Probably not, but nobody can promise that. Third-party clients are against
Discord's Terms of Service. Accounts that get banned are almost always spamming
or running automation. If losing your account would be a disaster, don't use
any third-party client.

## Privacy

Chatter talks straight to Discord. There's no Chatter server in between.

- You sign in by scanning a QR code with Discord's phone app, so Chatter never
  sees your password.
- Your login token stays in your system's credential store.
- No analytics, no crash reports, no update checks.

## Build it

### Windows

You need Rust (rustup), Visual Studio Build Tools with the C++ workload, LLVM
(for `LIBCLANG_PATH`) and NASM. From a Developer Command Prompt:

```bat
set LIBCLANG_PATH=C:\Program Files\LLVM\bin
set NASM=C:\Program Files\NASM\nasm.exe
cargo build --release --no-default-features --features desktop,discord
```

The app is `target\release\static.exe`.

### Linux

With [Nix](https://nixos.org):

```sh
nix develop path:./nix --command cargo run --locked --no-default-features --features desktop,discord
```

Without Nix, rustup installs the right Rust from `rust-toolchain.toml`, and
`packaging/arch/PKGBUILD` lists the libraries you need.

More detail is in [docs/building.md](docs/building.md).

## Credits

- **[Yapfast](https://yapfast.com)** ([github.com/yapfast](https://github.com/yapfast)),
  the client Chatter is built on. All credit for the core app goes to them.
- [Discord Userdoccers](https://docs.discord.food), for documenting Discord's
  user API.
- [Slint](https://slint.dev), [Skia](https://skia.org),
  [LiveKit](https://livekit.io), WebRTC,
  [matrix-rust-sdk](https://github.com/matrix-org/matrix-rust-sdk) and
  [DeepFilterNet](https://github.com/Rikorose/DeepFilterNet).

## License

GPL 3.0 or later, the same as Yapfast. See [LICENSE](LICENSE).

<sub>Chatter isn't affiliated with or endorsed by Discord Inc. Discord is a
trademark of Discord Inc.</sub>
