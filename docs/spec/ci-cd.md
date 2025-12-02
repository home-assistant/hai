# CI/CD (GitHub Actions)

## Principles

- **Immutable releases**: Once a version is released, it cannot be changed
- **Signed artifacts**: All release binaries are signed
- **Reproducible builds**: Same commit = same output
- **Multi-platform**: Build and test on macOS, Windows, Linux

---

## Workflow: Test (on every push/PR)

```yaml
# .github/workflows/test.yml
name: Test

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  test-rust:
    name: Rust Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      
      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov
      
      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri
      
      - name: Run tests with coverage
        run: cargo llvm-cov --lcov --output-path lcov.info
        working-directory: src-tauri
      
      - name: Upload Rust coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: src-tauri/lcov.info
          flags: rust
          token: ${{ secrets.CODECOV_TOKEN }}

  test-frontend:
    name: Frontend Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Run unit tests with coverage
        run: npm run test:unit -- --coverage
      
      - name: Run integration tests with coverage
        run: npm run test:integration -- --coverage
      
      - name: Upload frontend coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: coverage/lcov.info
          flags: frontend
          token: ${{ secrets.CODECOV_TOKEN }}

  test-e2e:
    name: E2E Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Install Playwright browsers
        run: npx playwright install --with-deps chromium
      
      - name: Install Tauri dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Run E2E tests
        run: npm run test:e2e
        env:
          HA_INSTALLER_MOCK: "true"
      
      - name: Upload test results
        uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: playwright-report
          path: playwright-report/
          retention-days: 7

  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Lint frontend
        run: npm run lint
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      
      - name: Rust format check
        run: cargo fmt --check
        working-directory: src-tauri
      
      - name: Rust clippy
        run: cargo clippy -- -D warnings
        working-directory: src-tauri
```

---

## Codecov Configuration

```yaml
# codecov.yml
coverage:
  precision: 2
  round: down
  status:
    project:
      default:
        target: auto
        threshold: 2%
    patch:
      default:
        target: 80%

flags:
  rust:
    paths:
      - src-tauri/src/
    carryforward: true
  frontend:
    paths:
      - src/
    carryforward: true

comment:
  layout: "reach,diff,flags,files"
  behavior: default
  require_changes: true
```

---

## Workflow: Release (on version tag)

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  # Validate tag matches version in Cargo.toml
  validate-version:
    name: Validate Version
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Check version consistency
        run: |
          TAG_VERSION="${GITHUB_REF#refs/tags/v}"
          CARGO_VERSION=$(grep '^version' src-tauri/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
          PKG_VERSION=$(node -p "require('./package.json').version")
          
          if [ "$TAG_VERSION" != "$CARGO_VERSION" ]; then
            echo "Tag version ($TAG_VERSION) doesn't match Cargo.toml ($CARGO_VERSION)"
            exit 1
          fi
          
          if [ "$TAG_VERSION" != "$PKG_VERSION" ]; then
            echo "Tag version ($TAG_VERSION) doesn't match package.json ($PKG_VERSION)"
            exit 1
          fi
          
          echo "Version $TAG_VERSION validated"

  # Run all tests before release
  test:
    name: Test
    needs: validate-version
    uses: ./.github/workflows/test.yml

  # Build for each platform
  build:
    name: Build (${{ matrix.os }})
    needs: test
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: hai_linux-x64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: hai_macos-x64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: hai_macos-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: hai_windows-x64

    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Install dependencies (Linux)
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev
      
      - name: Install frontend dependencies
        run: npm ci
      
      - name: Build frontend
        run: npm run build
      
      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
          # macOS signing
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        with:
          tagName: v__VERSION__
          releaseName: 'Home Assistant Installer v__VERSION__'
          releaseBody: 'See the assets below to download and install.'
          releaseDraft: true
          prerelease: false
          args: --target ${{ matrix.target }}
      
      - name: Generate checksums
        shell: bash
        run: |
          cd src-tauri/target/${{ matrix.target }}/release/bundle
          find . -type f \( -name "*.dmg" -o -name "*.app.tar.gz" -o -name "*.msi" -o -name "*.exe" -o -name "*.deb" -o -name "*.AppImage" \) -exec sha256sum {} \; > checksums.txt
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: |
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.dmg
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.app.tar.gz
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.msi
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.exe
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.deb
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.AppImage
            src-tauri/target/${{ matrix.target }}/release/bundle/**/checksums.txt
          retention-days: 5
      
      - name: Determine release type
        run: |
          VERSION="${GITHUB_REF#refs/tags/v}"
          if [[ "$VERSION" == *"-beta"* ]] || [[ "$VERSION" == *"-rc"* ]]; then
            echo "PRERELEASE=true" >> $GITHUB_ENV
          else
            echo "PRERELEASE=false" >> $GITHUB_ENV
          fi

  # Create GitHub release with all artifacts
  publish:
    name: Publish Release
    needs: build
    runs-on: ubuntu-latest
    permissions:
      contents: write
      id-token: write  # Required for cosign keyless signing
    steps:
      - uses: actions/checkout@v4
      
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts
      
      - name: Install cosign
        uses: sigstore/cosign-installer@v3
      
      - name: Generate release checksums
        run: |
          cd artifacts
          find . -type f \( -name "*.dmg" -o -name "*.app.tar.gz" -o -name "*.msi" -o -name "*.exe" -o -name "*.deb" -o -name "*.AppImage" \) -exec sha256sum {} \; | sort > SHA256SUMS.txt
      
      - name: Sign artifacts with cosign
        run: |
          cd artifacts
          # Sign each artifact with keyless signing (uses GitHub OIDC)
          for file in $(find . -type f \( -name "*.dmg" -o -name "*.app.tar.gz" -o -name "*.msi" -o -name "*.exe" -o -name "*.deb" -o -name "*.AppImage" \)); do
            cosign sign-blob --yes --output-signature "${file}.sig" --output-certificate "${file}.pem" "${file}"
          done
          # Also sign the checksums file
          cosign sign-blob --yes --output-signature SHA256SUMS.txt.sig --output-certificate SHA256SUMS.txt.pem SHA256SUMS.txt
      
      - name: Create release
        uses: softprops/action-gh-release@v1
        with:
          draft: true
          generate_release_notes: true
          prerelease: ${{ env.PRERELEASE }}
          files: |
            artifacts/**/*.dmg
            artifacts/**/*.app.tar.gz
            artifacts/**/*.msi
            artifacts/**/*.exe
            artifacts/**/*.deb
            artifacts/**/*.AppImage
            artifacts/**/*.sig
            artifacts/**/*.pem
            artifacts/SHA256SUMS.txt
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

## Cosign Verification

Users can verify the authenticity of downloads using cosign:

```bash
# Install cosign
# macOS: brew install cosign
# Linux: See https://docs.sigstore.dev/cosign/installation/

# Verify a downloaded artifact
cosign verify-blob \
  --signature hai_macos-arm64.dmg.sig \
  --certificate hai_macos-arm64.dmg.pem \
  --certificate-identity-regexp "https://github.com/home-assistant/hai/" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  hai_macos-arm64.dmg

# Verify the checksums file
cosign verify-blob \
  --signature SHA256SUMS.txt.sig \
  --certificate SHA256SUMS.txt.pem \
  --certificate-identity-regexp "https://github.com/home-assistant/hai/" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  SHA256SUMS.txt
```

Cosign keyless signing uses GitHub's OIDC identity, so signatures are tied to:
- The specific GitHub repository
- The GitHub Actions workflow that created them
- The exact commit and workflow run

No private keys to manage or rotate.

---

## Immutable Release Protections

1. **Tag protection rules** (GitHub Settings > Branches > Tag protection):
   - Pattern: `v*`
   - Prevents deletion or force-push of version tags

2. **Branch protection on main**:
   - Require PR reviews
   - Require status checks (all tests must pass)
   - No force pushes

3. **Release process**:
   - Releases are created as drafts
   - Manual review before publishing
   - Once published, artifacts cannot be modified

4. **Checksums**:
   - SHA256 checksums for all artifacts
   - Published with each release
   - Users can verify download integrity

5. **Cosign signatures**:
   - All artifacts signed using Sigstore cosign
   - Keyless signing via GitHub OIDC (no secrets to manage)
   - Signatures prove artifacts were built by the official CI pipeline
   - Publicly verifiable via Sigstore transparency log

---

## Required Secrets

| Secret | Description |
|--------|-------------|
| `CODECOV_TOKEN` | Codecov upload token (from codecov.io) |
| `TAURI_SIGNING_PRIVATE_KEY` | Key for signing Tauri updates |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password for signing key |
| `APPLE_CERTIFICATE` | Base64-encoded .p12 certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Certificate password |
| `APPLE_SIGNING_IDENTITY` | e.g., "Developer ID Application: Open Home Foundation" |
| `APPLE_ID` | Apple ID email for notarization |
| `APPLE_PASSWORD` | App-specific password |
| `APPLE_TEAM_ID` | Apple Developer Team ID |

---

## Release Process

1. Update version in `package.json` and `src-tauri/Cargo.toml`
2. Create PR with version bump
3. Merge to main after review
4. Create and push tag: `git tag v1.0.0 && git push origin v1.0.0`
5. CI builds and creates draft release
6. Review draft release and artifacts
7. Publish release (makes it immutable)

---

## Distribution

| Platform | Distribution Method |
|----------|---------------------|
| macOS | Direct download (.dmg), Homebrew Cask |
| Windows | Direct download (.msi/.exe), Microsoft Store |
| Linux | Direct download (.AppImage), Flathub, possibly .deb/.rpm |

Build automation via GitHub Actions to produce all artifacts on release.
