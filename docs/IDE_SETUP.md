<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# IDE Setup for anytag-backend

## VS Code

Recommended extensions for Rust development with mise + rustup:

- **rust-analyzer** – Rust language support (uses the rustup toolchain pinned in [`rust-toolchain.toml`](../rust-toolchain.toml))
- **CodeLLDB** – Debugging support
- **mise** – Environment/tool version management ([`hverlin.mise-vscode`](https://marketplace.visualstudio.com/items?itemName=hverlin.mise-vscode))
- **DockerDX** – Docker integration
- **Even Better TOML** – TOML file support (`Cargo.toml`, `mise.toml`, `diesel.toml`)
- **YAML** – YAML file support
- **Coverage Gutters** – Test coverage highlighting in the editor
- **Nix IDE** + **direnv** – only if you use the deprecated Nix environment (`flake.nix`)

### mise VS Code extension (recommended)

Install the **mise** extension and enable **`mise.configureExtensionsAutomatically`**
(see [mise docs](https://mise.jdx.dev/ide/vscode.html)). With this option on, mise:

- activates automatically when you open the project — no `mise activate` in your
  shell profile and no `mise x -- ...` prefix required;
- makes `diesel` and `reuse` available directly in the integrated VS Code
  terminal. You can run `cargo`, `diesel`, `reuse`, or `just` without wrapping
  them manually;
- ensures rust-analyzer's **Run (test)** / **Debug (test)** code lenses launch with
  the correct environment — `.env` is loaded and `DATABASE_URL` is constructed by
  [mise.toml](../mise.toml) — so tests that need PostgreSQL run without additional
  setup.

If you prefer not to use the extension, you can instead activate mise manually per
terminal:

```bash
eval "$(mise activate)"
```

or use the `just` wrappers (`just cargo …`, `just diesel …`, `just test`) which
always run through `mise x -- …`.

### Viewing Test Coverage

Test coverage is measured with [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov)
(install once via `cargo install cargo-llvm-cov`). The **Coverage Gutters** extension
(listed in [`.vscode/extensions.json`](../.vscode/extensions.json)) renders the coverage
directly in the editor.

1. Generate an LCOV report (the format Coverage Gutters reads):

   ```bash
   cargo llvm-cov --lcov --output-path lcov.info
   ```

2. In VS Code, run **Coverage Gutters: Watch** (or **Coverage Gutters: Display Coverage**) from the command palette.

3. Alternatively, view coverage without the editor:

   ```bash
   cargo llvm-cov          # Coverage summary in the terminal
   cargo llvm-cov --open   # HTML report in the browser
   ```

`lcov.info` is a generated artifact and is listed in `.gitignore`, so it is never committed.

## Zed

Zed has built-in Rust support via rust-analyzer. For mise + rustup development:

1. Install the **mise** extension from the extensions panel (or run `mise activate`
   in your shell profile)
2. Open the project — Zed will pick up the rustup toolchain from
   [`rust-toolchain.toml`](../rust-toolchain.toml)
3. Enable **mise** support in settings if using the extension

(The previous Nix/direnv flow — nix extension + direnv support — still works but is
deprecated.)

## IntelliJ/CLion

1. Install the "Rust" plugin
2. Open project — the IDE uses the rustup toolchain pinned in `rust-toolchain.toml`
   (configure it in Settings → Language & Frameworks → Rust if needed)
3. Set environment variables in run configurations — or point the IDE's environment
   at a mise-activated shell so `DATABASE_URL` is present

## See Also

- [Development Guide](./DEVELOPMENT.md) — Development workflow and common tasks
- [Troubleshooting](./TROUBLESHOOTING.md) — Common issues and solutions
