# Red Team Testing Suite

Comprehensive security testing framework for the Intent Segregation Cybersecurity Architecture. This module simulates real-world LLM attacks to verify defense mechanisms, implementing attack mechanisms from November 2025 state-of-the-art research.

**Report Date:** November 28, 2025
**Target System:** Intent Segregation Cybersecurity Architecture
**Scope:** Prompt injection attacks, jailbreak techniques, indirect injections, adaptive attacks
**Status:** Phase 1 implementation in progress

## 🎯 Overview

### Key Findings

**Attack Landscape (Nov 2025):**
- Prompt injection remains OWASP #1 risk
- Attack surface includes indirect injections, multimodal attacks, agent-to-agent exploitation
- Multi-turn jailbreaks have 70%+ success rate on undefended systems

**State-of-the-Art Defenses:**
- Task Shield: 2.07% ASR (vs 26-41% baseline)
- SmoothLLM: <1% ASR (vs 50-90% undefended)
- Microsoft Prompt Shields: 15-20% detection rate

**Your Architecture Alignment:**
| Component | Defense Mechanism | Status |
|-----------|------------------|--------|
| Vault of the Forbidden Cant | Zero-trust testing | ✓ Equivalent to Prompt Shields |
| Council of Oracular Cogitors | Multi-parser ensemble | ✓ Reduces transfer attacks |
| The Voting Engine | Consensus voting | ✓ Similar to SmoothLLM |
| Judicator of Concordance | Policy enforcement | ✓ Similar to Task Shield |
| Typed Execution | No free-form LLM calls | ✓ Critical advantage |

### Attack Categories Implemented

- **Direct Prompt Injection**: HashJack, Unicode obfuscation, semantic substitution, dual intention escape
- **Indirect Injection**: Website content, email, multi-agent cascade, multimodal attacks
- **Jailbreak Techniques**: Roleplay, multi-turn progression, weak-to-strong transfer
- **Consensus-Breaking**: Parser-specific exploits, voting confusion
- **Adaptive Attacks**: RL-based (32 sessions × 5 rounds), search-based (1000 variants), cascade attacks
- **Domain Scenarios**: B2B, customer service, phone tree, financial, healthcare, e-commerce

## 📂 Structure

```
redteam/
├── README.md                          # This file
├── BENCHMARKS.md                      # Comprehensive benchmark & threat model
├── PAYLOAD_LIBRARY.md                 # Attack payload documentation
├── benchmarks/
│   ├── metrics.rs                     # Metric measurement (ASR, FRR, latency, etc.)
│   ├── dashboard.rs                   # Real-time metrics dashboard & reporting
│   ├── runners.rs                     # Test orchestration & benchmark execution
│   └── datasets.rs                    # Benchmark dataset loaders (BIPIA, TaskTracker, AgentDojo, ASB)
├── attacks/
│   ├── direct_injection/              # Phase 1 attacks
│   ├── indirect_injection/            # Phase 2 attacks
│   ├── jailbreaks/                    # Phase 3 attacks
│   ├── consensus_breaking/            # Phase 4 attacks
│   └── adaptive/                      # Phase 5 attacks
├── payloads/
│   ├── direct_injection.txt           # 100+ direct injection payloads
│   ├── indirect_injection.txt         # 150+ indirect injection payloads
│   ├── jailbreaks.txt                 # 200+ jailbreak variants
│   └── [additional payload files]
├── scenarios/
│   ├── b2b_consulting.rs              # B2B consulting platform attacks
│   ├── customer_service.rs            # Customer service portal attacks
│   ├── phone_tree.rs                  # IVR/Phone tree attacks
│   ├── financial.rs                   # Banking/payment systems
│   ├── healthcare.rs                  # Healthcare system attacks
│   └── ecommerce.rs                   # E-commerce platform attacks
└── analysis/
    ├── attack_success_rate.rs         # ASR analysis & reporting
    ├── defense_effectiveness.rs       # Defense layer analysis
    └── report_generator.rs            # Report generation
```

## 🚀 Quick Start

### Run All Red Team Tests
```bash
cargo test --test redteam
```

### Run by Phase
```bash
# Phase 1: Direct Injection
cargo test --test redteam phase_1_direct_injection

# Phase 2: Indirect Injection
cargo test --test redteam phase_2_indirect_injection

# Phase 3: Jailbreaks
cargo test --test redteam phase_3_jailbreaks

# Phase 4: Consensus-Breaking
cargo test --test redteam phase_4_consensus_breaking

# Phase 5: Adaptive Attacks
cargo test --test redteam phase_5_adaptive
```

### Run by Attack Type
```bash
# Direct injection attacks
cargo test --test redteam hashjack
cargo test --test redteam unicode_obfuscation
cargo test --test redteam semantic_substitution

# Jailbreak attacks
cargo test --test redteam roleplay
cargo test --test redteam multi_turn
cargo test --test redteam weak_to_strong

# Adaptive attacks
cargo test --test redteam rl_based_attack
cargo test --test redteam search_based_attack

# Domain scenarios
cargo test --test redteam b2b_consulting
cargo test --test redteam customer_service
cargo test --test redteam phone_tree
cargo test --test redteam financial
cargo test --test redteam healthcare
cargo test --test redteam ecommerce
```

### Run with Metrics Output
```bash
# Show all output including metrics
cargo test --test redteam -- --nocapture

# Show metrics only
cargo test --test redteam measure_metrics -- --nocapture
```

### Run Benchmark Datasets
```bash
# BIPIA benchmark (indirect injection - 3K samples)
cargo test --test redteam bipia_evaluation

# TaskTracker benchmark (31K samples - statistical power)
cargo test --test redteam tasktracker_evaluation

# AgentDojo benchmark (100+ scenarios, 4 domains)
cargo test --test redteam agentdojo_evaluation

# ASB benchmark (10 domains, 400+ tools, 27 attack methods)
cargo test --test redteam asb_evaluation
```

## 📊 Metrics Explained

### Core Security Metrics

| Metric | Target | Description |
|--------|--------|-------------|
| **ASR** | <5% | Attack Success Rate - % of attacks that bypass defenses |
| **FRR** | <10% | False Refusal Rate - % of benign requests incorrectly rejected |
| **Vault Detection** | >95% | % of prompt injections detected by Vault |
| **Parser Agreement** | >95% | % of benign requests where ≥2 parsers agree |
| **Voting Conflict Det.** | >85% | % of attacks triggering conflict escalation |
| **Policy Enforcement** | >99% | % of policy violations correctly blocked |

### Utility Metrics

| Metric | Target | Description |
|--------|--------|-------------|
| **Clean Utility** | >75% | % of benign tasks completed successfully |
| **Utility Under Attack** | >65% | % of benign tasks successful when under attack |

### NEW Adaptive Metrics (November 2025)

| Metric | Target | Description |
|--------|--------|-------------|
| **Adaptive ASR(k=100)** | <15% | ASR after 100 optimization iterations |
| **k-Robustness** | <1.5x | AAR(100) ≤ AAR(0) × 1.5 |
| **Query Budget** | >100 | Queries needed per successful attack |

### Performance Metrics

| Metric | Target | Description |
|--------|--------|-------------|
| **Latency (avg)** | <2s | Average response time |
| **Latency (P95)** | <3s | 95th percentile response time |
| **Throughput** | >10 req/s | Requests processed per second |
| **Token Overhead** | <3x | Token consumption vs baseline |

## 📖 Understanding Results

### Attack Success Rate (ASR)

```
TIER 1 (Competitive):     <5%   (vs 26-41% baseline)
TIER 2 (Publication):     <2%   (vs 2.07% Task Shield)
TIER 3 (Best-in-Class):   <1%   (vs <1% SmoothLLM)
```

**What it means:**
- Low ASR = More attacks blocked
- If ASR = 2.3%, then 97.7% of attacks were blocked

### Adaptive Attack Success Rate (AAR)

```
AAR(0):   Initial ASR on unoptimized attacks
AAR(100): ASR after 100 adaptive attempts
k-robust if: AAR(100) ≤ AAR(0) × 1.5
```

**What it means:**
- Measures resistance to attackers who learn and adapt
- Defense should not get significantly easier to bypass with optimization
- If AAR(0)=5% and AAR(100)=12%, that's 2.4x increase (FAILS k-robustness)

### False Refusal Rate (FRR)

```
TIER 1: <10%  (vs 15-30% high-security baselines)
TIER 2: <8%
TIER 3: <5%
```

**What it means:**
- % of legitimate requests wrongly rejected
- Balance between security and usability
- Too high FRR = defense is too aggressive

### Parser Agreement

```
High agreement (>95%):   Confidence in intent extraction
Low agreement (75-95%):  Potential ambiguity
Conflict (<75%):         Suspicious, escalate to human
```

**What it means:**
- When OpenAI, DeepSeek, and Claude agree on intent: HIGH CONFIDENCE
- If 1-2 parsers disagree: Worth escalating
- If all disagree: DEFINITELY suspicious

## 🔴 Red Flag Indicators

If you see these metrics, investigate immediately:

| Metric | Red Flag | Action |
|--------|----------|--------|
| ASR | >10% | Defense layer failing |
| FRR | >20% | Defense too aggressive |
| Vault Detection | <80% | Vault bypasses exist |
| Parser Agreement | <85% | Insufficient consensus |
| Adaptive ASR | >AAR(0) × 2.0 | Not k-robust |
| Latency | >3s avg | Performance regression |

## 📚 References

### Attack Payloads
All payloads derived from peer-reviewed academic papers:

- **HashJack** (Nov 2025): Cato Networks research
- **LatentBreak** (Oct 2025): arXiv:2510.08604
- **Invisible Unicode** (May 2025): Keysight security research
- **DIE** (Jan 2025): Dual intention escape attacks
- **ServiceNow Agents** (Nov 2025): Enterprise multi-agent exploitation
- **CaMeL** (Mar 2025): arXiv:2503.18813 (defense reference)
- **DefensiveTokens** (Jul 2025): arXiv:2507.07974 (defense reference)
- **"The Attacker Moves Second"** (Oct 2025): arXiv:2510.09023 (adaptive attacks)

### Benchmarks
- **BIPIA**: Indirect Prompt Injection Attacks (Yi et al., KDD '25)
- **TaskTracker**: 31K-sample injection dataset (Abdelnabi et al., 2025)
- **AgentDojo**: Agentic security benchmark (Google DeepMind)
- **ASB**: Agent Security Bench (ICLR 2025)

## 🧬 Test Organization

Each attack test follows this pattern:

```rust
#[tokio::test]
async fn test_attack_name() {
    // 1. Create malicious input
    let payload = "attack payload here";

    // 2. Submit through system
    let result = system.process_request(&payload).await;

    // 3. Verify defense
    assert!(result.was_blocked() || result.was_escalated());

    // 4. Metrics collected automatically
    // - Time taken
    // - Parser agreement
    // - Voting result
    // - Detection method
}
```

## 🤝 Contributing New Attacks

To add a new attack:

1. Create test file in appropriate directory (`attacks/phase_X/`)
2. Implement attack logic
3. Add payloads to `payloads/` directory
4. Update metrics collection
5. Document in this README
6. Add to appropriate test suite
7. Update `WorkInProgress.md` checkbox

## 🔐 Safety & Ethics

**This is defensive security research only.**

- ✅ Testing OUR OWN system's defenses
- ✅ Helping improve security
- ✅ Based on published academic research
- ✅ All payloads documented and cited
- ❌ NOT for attacking other systems
- ❌ NOT for malicious purposes
- ❌ NOT for unauthorized testing

## 📝 Documentation

- **[BENCHMARKS.md](./BENCHMARKS.md)** - Comprehensive threat model & benchmarks
- **[PAYLOAD_LIBRARY.md](./PAYLOAD_LIBRARY.md)** - Attack payload documentation
- **[WorkInProgress.md](../../WorkInProgress.md)** - Implementation tracking

## ⚙️ Configuration

See `.env.example` for:
- LLM model selections (OpenAI, DeepSeek, Claude)
- Rate limiting settings
- Timeout configurations
- Logging levels

## 📞 Support

If tests fail or behavior seems wrong:

1. Check `WorkInProgress.md` for known issues
2. Review attack-specific documentation in `BENCHMARKS.md`
3. Check parser configuration in `.env`
4. Verify all LLM API keys are set
5. Run `cargo test --test redteam -- --nocapture` for detailed output

---

**Last Updated:** November 28, 2025
**Status:** Phase 1 Implementation in Progress
**Target Completion:** Week 9 (December 26, 2025)
