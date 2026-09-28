# Release readiness

KMJ Desktop Commander is ready to be promoted from prerelease to stable only when every gate below is verified.

## Installer gates

- Windows x64 NSIS installer builds and passes artifact existence + SHA-256 verification.
- Linux x64 AppImage and DEB both build and pass artifact existence, file-type and SHA-256 verification.
- macOS Apple Silicon DMG builds and passes artifact existence, file-type and SHA-256 verification.
- macOS Intel DMG builds and passes artifact existence, file-type and SHA-256 verification.
- Frontend typecheck/build and Rust fmt/clippy/tests pass on the same source revision.
- Installer artifacts are produced from a tag whose version matches the application version.
- The release includes SHA256SUMS.txt.

## Trust gates for stable public distribution

- Windows Authenticode signing is enabled with a protected CI secret/certificate or trusted signing service.
- macOS Developer ID signing and Apple notarization are enabled.
- macOS stapling verification passes for the distributed DMGs.
- Update metadata is signed before automatic updates are enabled.
- No SSH private key, API token, signing key or customer source is embedded in an installer.
- Fresh-machine smoke install/uninstall is verified on supported OS versions.

Until the trust gates are complete, GitHub tag builds must remain prereleases rather than being advertised as stable production installers.
