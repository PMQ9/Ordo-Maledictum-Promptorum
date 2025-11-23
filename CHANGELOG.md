# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Detailed ASCII architecture diagram based on actual code implementation analysis
- Security analysis section with verified safe-by-design principles
- Complete 8-stage security pipeline documentation showing data flow through all modules
- Trust boundary documentation showing where user input is segregated from execution
- Areas for improvement section identifying 6 specific security enhancements needed

### Changed
- ARCHITECTURE.md now includes actual implementation details verified from source code
- Reorganized documentation to show both actual implementation and high-level overview

### Added
- Created CLAUDE.md documentation file for Claude Code instances
  - Comprehensive build, test, and run commands
  - High-level architecture overview including 8-stage validation pipeline
  - Module dependency graph and key design patterns
  - Security constraints and development workflows
  - Configuration guidance and common development tasks
  - Change documentation section requiring updates to CHANGELOG.md
- Created CHANGELOG.md following Keep a Changelog format

### Changed

### Deprecated

### Removed
- Removed all CI/CD workflows (ci.yml, docker.yml, docs.yml, release.yml, security.yml)
  - Workflows were not functioning properly and consuming excessive resources
  - CI/CD documentation files removed (CICD_SETUP_SUMMARY.md, .github/WORKFLOWS_QUICK_REFERENCE.md)
  - CI/CD references removed from CLAUDE.md, CONTRIBUTING.md, and PR template
  - Can be re-added later once issues are resolved

### Fixed

### Security
