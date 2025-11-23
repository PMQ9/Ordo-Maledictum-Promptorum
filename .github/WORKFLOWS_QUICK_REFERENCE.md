# GitHub Actions Workflows - Quick Reference

## Workflows Overview

| Workflow | Trigger | Purpose | Duration |
|----------|---------|---------|----------|
| CI | Push, PR | Build, test, lint | ~15-20 min |
| Security | Push (main), PR, Daily | Security scans | ~10-15 min |
| Docker | Push (main), Tags | Container build & push | ~8-12 min |
| Release | Tags (v*.*.*) | Build & publish release | ~30-45 min |
| Docs | Push (main), PR | Generate & deploy docs | ~5-10 min |

## Quick Commands

### Local Development

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features

# Security audit
cargo audit

# Build release
cargo build --release
```

### Frontend

```bash
cd frontend
npm install
npm run lint
npm run build
```

### Docker

```bash
# Build locally
docker build -t intent-segregation:local .

# Run
docker-compose up
```

### Release Process

```bash
# Create and push tag
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

## Workflow Status Checks

### Required for PR Merge
- ✅ Lint
- ✅ Build (debug & release)
- ✅ Test (stable Rust)
- ✅ Frontend Build
- ✅ Security Audit

### Optional (Informational)
- Test (beta/nightly)
- Code Coverage
- Supply Chain Security

## Common Issues

### Tests Failing?
1. Check database connection (PostgreSQL required)
2. Update dependencies: `cargo update`
3. Check environment variables

### Clippy Warnings?
```bash
cargo clippy --fix --all-targets --all-features
```

### Format Issues?
```bash
cargo fmt --all
```

### Security Vulnerabilities?
```bash
cargo audit
cargo audit --deny warnings
```

## Secrets Required

| Secret | Purpose | Required |
|--------|---------|----------|
| GITHUB_TOKEN | Auto-provided | Yes |
| CARGO_REGISTRY_TOKEN | Publish to crates.io | Optional |
| CODECOV_TOKEN | Coverage reporting | Optional |

## File Structure

```
.github/
├── workflows/
│   ├── ci.yml              # Main CI pipeline
│   ├── security.yml        # Security scanning
│   ├── docker.yml          # Container builds
│   ├── release.yml         # Release automation
│   └── docs.yml            # Documentation
├── ISSUE_TEMPLATE/
│   ├── bug_report.md
│   ├── feature_request.md
│   └── config.yml
├── CODEOWNERS              # Code ownership
├── PULL_REQUEST_TEMPLATE.md
├── SECURITY.md             # Security policy
└── dependabot.yml          # Dependency updates
```

## Useful Links

- [Actions Tab](../../actions) - View workflow runs
- [Security Tab](../../security) - Security alerts
- [Insights](../../pulse) - Repository insights
- [Packages](../../pkgs/container/intent-segregation) - Container registry

## Support

- 📖 Full documentation: [CICD_SETUP_SUMMARY.md](../CICD_SETUP_SUMMARY.md)
- 🐛 Issues: [New Issue](../../issues/new/choose)
- 💬 Discussions: [Discussions](../../discussions)
- 🔒 Security: [Report Vulnerability](../../security/advisories/new)
