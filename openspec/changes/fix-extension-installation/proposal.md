# Change: Fix Extension Installation Hang

## Why

The REST Client extension experiences installation hangs when users try to install it via "Install Dev Extension" in Zed. The root cause is a misconfiguration in `Cargo.toml` where the library is compiled with both `cdylib` and `lib` crate types. When Zed's extension system attempts to load a WASM module that includes the `lib` crate type alongside `cdylib`, it causes the installation process to hang indefinitely (>60 seconds, often requiring force-quit).

This creates a critical blocker for users attempting to install and use the extension, resulting in a poor first-run experience and preventing adoption.

## What Changes

- **Cargo.toml**: Remove `"lib"` from `crate-type` array, keeping only `["cdylib"]` for WASM compilation
- **Build validation**: Ensure the extension WASM binary compiles correctly with the single crate type
- **Installation workflow**: Verify that the fixed configuration allows installation to complete in <60 seconds

This is a **configuration fix** that resolves the installation hang without changing any functional behavior of the extension itself.

## Impact

- **Affected specs**: `extension-installation` (new capability spec)
- **Affected code**: 
  - `Cargo.toml` (line 11: `crate-type` field)
  - No source code changes required
- **User impact**: 
  - Immediate resolution of installation hang issue
  - Installation time reduced from indefinite hang to 30-60 seconds
  - Enables users to successfully install and use the extension
- **Breaking changes**: None (this is a bug fix restoring intended behavior)
- **Testing requirements**:
  - Manual verification of "Install Dev Extension" workflow
  - Confirmation that WASM binary builds successfully
  - Validation that extension loads and activates in Zed

## Success Criteria

- Extension installs successfully via "Install Dev Extension" in <60 seconds
- WASM binary compiles without errors using only `cdylib` crate type
- Extension appears in Zed Extensions panel with "Installed" or "Enabled" status
- HTTP language support activates correctly for `.http` and `.rest` files