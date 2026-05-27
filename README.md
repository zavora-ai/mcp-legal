# Legal Reference MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-legal.svg)](https://crates.io/crates/mcp-legal)
[![Docs.rs](https://docs.rs/mcp-legal/badge.svg)](https://docs.rs/mcp-legal)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

Governed, global legal reference data for [ADK-Rust Enterprise](https://enterprise.adk-rust.com) agents. Provides 24 MCP tools across 14 jurisdictions — **retrieves and structures legal information without providing legal advice**.

## Key Principles

- **No legal advice** — every response carries `not_legal_advice: true`. The server retrieves; lawyers interpret.
- **Citation required** — all results include source URLs, jurisdiction tags, and retrieval timestamps.
- **Human review recommended** — every response flags `human_review_recommended: true`.
- **Multi-jurisdictional** — 14 countries across Americas, Europe, Asia-Pacific, and global sanctions.
- **Governed access** — all tools are read-only with governance gates for audit compliance.

## Tools

| Tool | Jurisdiction | Purpose | Risk Class |
|------|:---:|---------|:---:|
| `search_cases` | US | Search 117M+ court opinions | Read-only |
| `get_case` | US | Get opinion by cluster ID | Read-only |
| `search_federal_register` | US | Search regulations, proposed rules | Read-only |
| `get_federal_register_document` | US | Get regulation by document number | Read-only |
| `search_uk_legislation` | UK | Search Acts, Statutory Instruments | Read-only |
| `get_uk_legislation` | UK | Get legislation by identifier | Read-only |
| `search_eu_legislation` | EU | Search directives, regulations | Read-only |
| `get_eu_document` | EU | Get document by CELEX number | Read-only |
| `get_german_law` | DE | Get law in English translation | Read-only |
| `get_swiss_law` | CH | Get law by ELI identifier | Read-only |
| `search_swedish_legislation` | SE | Search Swedish laws (JSON API) | Read-only |
| `get_norwegian_law` | NO | Get law in English | Read-only |
| `get_korean_law` | KR | Get law in English (KLRI) | Read-only |
| `search_australian_legislation` | AU | Search Acts, instruments | Read-only |
| `search_japanese_laws` | JP | Search laws via e-Gov | Read-only |
| `get_canadian_law` | CA | Get full statute text (XML) | Read-only |
| `get_irish_legislation` | IE | Get Act by year/number | Read-only |
| `get_nz_legislation` | NZ | Get Act by identifier | Read-only |
| `screen_entity` | Global | Sanctions screening (200+ lists) | Read-only |
| `search_sanctions` | Global | Search sanctions by keyword | Read-only |
| `get_sanctions_record` | Global | Get entity record by ID | Read-only |
| `list_supported_jurisdictions` | — | List jurisdictions and sources | Read-only |
| `list_supported_sources` | — | List data sources | Read-only |
| `get_coverage_status` | — | Coverage gaps and roadmap | Read-only |

## Jurisdictions

| Region | Countries | Sources |
|--------|-----------|---------|
| Americas | 🇺🇸 United States, 🇨🇦 Canada | CourtListener, Federal Register, Justice Laws XML |
| Europe | 🇬🇧 UK, 🇪🇺 EU, 🇩🇪 Germany, 🇨🇭 Switzerland, 🇮🇪 Ireland, 🇸🇪 Sweden, 🇳🇴 Norway | Legislation.gov.uk, EUR-Lex, gesetze-im-internet.de, Fedlex, Irish Statute Book, Riksdagen, Lovdata |
| Asia-Pacific | 🇦🇺 Australia, 🇯🇵 Japan, 🇰🇷 South Korea, 🇳🇿 New Zealand | legislation.gov.au, e-Gov, KLRI, NZ Legislation |
| Global | 200+ sanctions lists | Open Sanctions |

## Installation

### Build from source

```bash
git clone https://github.com/zavora-ai/mcp-legal
cd mcp-legal
cargo build --release
```

### From crates.io

```bash
cargo install mcp-legal
```

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "legal": {
      "command": "mcp-legal"
    }
  }
}
```

### Cursor

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "legal": {
      "command": "mcp-legal"
    }
  }
}
```

### Kiro

Add to `.kiro/settings/mcp.json`:

```json
{
  "mcpServers": {
    "legal": {
      "command": "mcp-legal"
    }
  }
}
```

No API keys or environment variables required. All sources are free and public.

## Response Schema

Every tool returns a normalized response:

```json
{
  "source": "courtlistener",
  "source_type": "case_law",
  "jurisdiction": "US",
  "title": "In re Equifax, Inc., Customer Data Sec. Breach Litig.",
  "citation": "289 F. Supp. 3d 1322",
  "source_url": "https://www.courtlistener.com/opinion/...",
  "retrieved_at": "2026-05-27T05:00:00Z",
  "published_at": "2017-12-06",
  "version_status": "current",
  "not_legal_advice": true,
  "human_review_recommended": true,
  "metadata": {
    "court": "United States Judicial Panel on Multidistrict Litigation",
    "cite_count": 55
  }
}
```

Sanctions responses additionally include:

```json
{
  "match_confidence": 0.95,
  "matched_fields": ["name", "alias"],
  "aliases": ["..."],
  "datasets": ["us_ofac_sdn", "eu_sanctions"]
}
```

## Governance Model

### Tool Labels

All tools carry these governance labels:

```text
read_only
legal_reference
citation_required
jurisdiction_tag_required
source_metadata_required
retrieval_timestamp_required
no_legal_advice
human_review_recommended
```

### Sanctions Labels

```text
sanctions_sensitive
false_positive_possible
compliance_review_required
```

### MCP Server Manifest

```toml
server_id = "mcp_legal"
display_name = "Legal"
version = "1.4.0"
domain = "legal"
risk_level = "medium"
writes_allowed = "none"
governance_gates = ["no_legal_advice", "citation_required", "human_review_recommended"]
```

## Common Identifiers

### UK Legislation
| Identifier | Act |
|-----------|-----|
| `ukpga/2018/12` | Data Protection Act 2018 |
| `ukpga/2006/46` | Companies Act 2006 |
| `ukpga/2010/15` | Equality Act 2010 |

### EU CELEX Numbers
| CELEX | Document |
|-------|----------|
| `32016R0679` | GDPR |
| `32024R1689` | AI Act |
| `32022R2065` | Digital Services Act |
| `32022R1925` | Digital Markets Act |

### German Law Codes
| Code | Law |
|------|-----|
| `bdsg` | Federal Data Protection Act |
| `bgb` | Civil Code |
| `stgb` | Criminal Code |
| `gg` | Basic Law (Constitution) |
| `hgb` | Commercial Code |

### Canadian Act Codes
| Code | Act |
|------|-----|
| `P-21` | Privacy Act |
| `C-46` | Criminal Code |
| `A-1` | Access to Information Act |

### South Korean KLRI hseq
| hseq | Act |
|------|-----|
| `53044` | Personal Information Protection Act |
| `46795` | IT Network Use Promotion Act |

## Safety and Disclaimer

> This server provides legal reference information and document retrieval only. It does not provide legal advice. Consult qualified counsel before relying on these materials for legal decisions.

The MCP returns machine-readable flags for downstream enforcement:

```json
{
  "not_legal_advice": true,
  "human_review_recommended": true,
  "citation_required": true,
  "jurisdiction_specific": true
}
```

## Roadmap

### v1.5 — Africa and India
- AfricanLII (Kenya, Nigeria, South Africa, Ghana, Uganda, Tanzania)
- Indian Kanoon (with API token)
- Laws.Africa (20+ African countries)

### v2.0 — Contract Extraction
- `extract_contract_terms`, `extract_parties`, `extract_dates`
- `extract_obligations`, `extract_clauses`, `extract_governing_law`
- `compare_clause_to_playbook`

### v2.1 — Evidence Packets
- `build_sanctions_evidence_packet`
- `find_citing_cases`
- Audit trail generation

## Documentation

| Document | Description |
|----------|-------------|
| [mcp-server.toml](mcp-server.toml) | ADK-Rust Enterprise registry manifest |
| [CHANGELOG.md](CHANGELOG.md) | Version history |

## Contributing

Contributions welcome. Priority areas:
- Additional jurisdiction connectors
- Citation normalization improvements
- Source freshness monitoring

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

Built with ❤️ by [Zavora AI](https://zavora.ai)

## Registry Compliance

This server implements the [ADK MCP SDK](https://crates.io/crates/adk-mcp-sdk) contract:

- **mcp-server.toml** — manifest declaring tools, risk classes, and governance gates
- **Normalized responses** — citations, jurisdiction tags, timestamps on every result
- **Governance gates** — `no_legal_advice`, `citation_required`, `human_review_recommended`
