# Ordo Maledictum Promptorum

<img width="75%" alt="Ordo Maledictum Promptorum" src="docs/images/project_title.png" />

Researching a system for preventing prompt injection by separating user intent from user content. This system treats all user inputs as potentially corrupted and uses a multi-layered defense strategy with sacrificial AI sentries and consensus voting.

## 1. Overview
This project implements an intent-first, schema-driven security architecture designed to mitigate prompt injection and unsafe LLM actions. The system separates:
- User Intent (what the user wants the system to do)
- User Content (text, documents, contextual details)

The architecture uses:

- Binahric Subversion Mantra (input prompt validation)
- Sacrificial AI sentries (The Penitent Cogitators) for input health checking
- Independent intent parsers (The Council of the Oracular Cogitors)
- The Voting Engine (consensus mechanism)
- An intent comparator (The Judicator of Concordance)
- The Arbiter of Purpose (trusted intent generation)
- The Oathbound Engine (execution via typed functions)
- An immutable audit ledger (The Chronicle of Allowed Thought)
- Optional human approval (The Overseer-Prime) for elevated-risk actions

This design is especially suitable for narrow, well-defined AI applications, such as B2B consulting automation, customer support tools, or workflow agents.

## 2. System Architecture
```
Binahric Subversion Mantra (User Input Prompt)
   │
   ├──► Vault of the Forbidden Cant
   │         │
   │         ├──► The Penitent Cogitators (3 Sacrificial AI Models)
   │         │
   │         └──► The Lexicanum Diagnostica (Health Monitor)
   │
   ├──► Council of the Oracular Cogitors (P1, P2, P3...)
   │         │
   │         └──► The Voting Engine
   │
   ├──► The Judicator of Concordance ◄── The Edict of the High Magister
   │
   ├─── if mismatch → Elevated Privilege Request → The Overseer-Prime
   │
   ├──► The Arbiter of Purpose
   │
   ├──► The Oathbound Engine
   │         │
   │         └──► The Chronicle of Allowed Thought (Immutable Ledger)
   │               └──► Adeptus Cogitatus Log Extract (Output)
   │
   └──► Response
```

<img width="125%" alt="Model Architecture" src="docs/images/model_architecture.png" />

 <!-- <img width="75%" alt="Screenshot 2025-11-22 163153" src="https://github.com/user-attachments/assets/256bcc99-0609-48a8-b166-75735769b1af" /> -->


## 3. Modules
### 3.1 Binahric Subversion Mantra

The raw user input prompt - treated as potentially corrupted and requiring validation before processing.

Characteristics:
- User-provided text input
- Assumes zero trust - all prompts considered suspicious
- Primary source of potential prompt injection attacks

Flows to: Vault of the Forbidden Cant for testing

### 3.2 Vault of the Forbidden Cant (Sacrificial Input Testing)

Purpose: Treat all inputs as potentially corrupted and test them on isolated AI sentries before proceeding.

Implementation:
- **The Penitent Cogitators**: 3 independent sacrificial LLM instances in an isolated sandbox
- **The Lexicanum Diagnostica**: Health monitoring system that probes the sacrificial models without touching them directly
- Models are tested with the user input to detect signs of corruption or attack patterns
- If models fail health checks, input is quarantined and escalated

Output: "safe" or "corrupted" determination.

### 3.3 Council of the Oracular Cogitors (Intent Parser Ensemble)

Several independent modules parse free-form user text into a structured intent JSON.

Parsers:

1. Deterministic Parser
  - Rule-based mapping for actions.
  - Keyword → enum resolution.
  - Zero hallucination; highest trust.
2. LLM Parser #1
  - Extracts action, topic, expertise, constraints.
  - Temperature set to 0.
  - Constrained to JSON schema.
3. LLM Parser #2
  - Same schema, but different model or system prompt.
  - Increases independence.

Output: structured JSON for each parser.

### 3.4 The Voting Engine

Compares outputs from all parsers to establish consensus.

Logic:

- If all parsers agree → high confidence.
- If small discrepancies → fallback to deterministic parser + request user confirmation.
- If major discrepancies → escalate to human approval.

Output: canonical parsed intent.

### 3.5 The Judicator of Concordance (Intent Comparator)

Compares:

- Parsed user intent vs
- The Edict of the High Magister (provider-defined allowed intents and capability config)

```
The Edict of the High Magister Example:

{
  "allowed_actions": ["find_experts", "summarize", "draft_proposal"],
  "allowed_expertise": ["ml", "embedded", "security"],
  "max_budget": 50000
}
```

The Judicator checks:

- action is allowed (enum)
- expertise subset is allowed
- parameters are within constraints
- topic is semantically similar to allowed domain

Decision:

- match → approve
- soft mismatch → require confirmation
- hard mismatch → block or escalate

### 3.6 The Arbiter of Purpose (Trusted Intent Generator)

Produces a canonical, sanitized, and signed JSON object that represents the approved user intent.

Guarantees:
- Only allowed fields appear.
- No raw user content is injected.
- content_refs are references to sanitized documents.

Example output:
```
{
  "action": "find_experts",
  "topic_id": "supply_chain_risk",
  "expertise": ["security"],
  "constraints": {
    "max_budget": 20000
  },
  "content_refs": ["doc_1321"]
}
```

### 3.7 The Oathbound Engine (Processing Engine)

Executes trusted intents via typed function calls—never raw prompts.

Implementation Requirements:

- All operations are strongly typed.
- Processing agents consume only trusted intents from The Arbiter of Purpose.
- No free-form LLM calls can execute privileged actions.

Example callable:
```
get_experts({ topic_id, expertise, max_budget })
```

### 3.8 The Chronicle of Allowed Thought (Intent Ledger)

Append-only log storing:
- User input
- Parsed intents from ensemble
- Comparator decisions
- Trusted intent
- Processing outputs
- Any privilege elevation events

Writable only; immutable once saved.

Great for:

- Auditing
- Forensics
- Explaining decisions
- Research evaluation

### 3.9 The Overseer-Prime (Human Supervision Module)

Triggered when:
- Intent mismatch
- Model disagreement
- High-risk actions
- Unusual parameter patterns

The Overseer-Prime receives:
- Raw user input
- Parsed JSON intents
- Diffs
- Explanation of mismatch

They can approve / deny / correct.

## 4. Example Folder Structure

```
.
├── core/
│   ├── parsers/
│   ├── comparator/
│   ├── intent_generator/
│   ├── malicious_detector/
│   ├── processing_engine/
│   ├── ledger/
│   └── supervision/
│
├── config/
│   ├── provider_config.json
│   └── schema.json
│
├── api/
│   └── server.py
│
├── tests/
│   └── redteam/
│
└── README.md
```
