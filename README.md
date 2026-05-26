# mcp-legal

[![Crates.io](https://img.shields.io/crates/v/mcp-legal.svg)](https://crates.io/crates/mcp-legal)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Global legal reference MCP server — case law, legislation, regulations, and sanctions screening across US, UK, EU, and 200+ sanctions lists. **14 tools** for legal research, compliance, and regulatory intelligence. All responses include citations, jurisdiction tags, and `not_legal_advice: true`.

> **This server provides legal reference information only. It does not provide legal advice. Consult qualified counsel before relying on these materials for legal decisions.**

## Installation

```bash
cargo install mcp-legal
mcp-legal  # No API keys required
```

## Tools (14)

### US Case Law (CourtListener)
| Tool | Description |
|------|-------------|
| `search_cases` | Search 117M+ US court opinions by keyword |
| `get_case` | Get specific opinion by cluster ID |

### UK Legislation
| Tool | Description |
|------|-------------|
| `search_uk_legislation` | Search UK Acts and Statutory Instruments |
| `get_uk_legislation` | Get legislation by identifier (e.g. `ukpga/2018/12`) |

### EU Legislation (EUR-Lex)
| Tool | Description |
|------|-------------|
| `search_eu_legislation` | Search EU regulations, directives |
| `get_eu_document` | Get document by CELEX number (e.g. `32016R0679` for GDPR) |

### US Regulations (Federal Register)
| Tool | Description |
|------|-------------|
| `search_federal_register` | Search regulations, proposed rules, notices |
| `get_federal_register_document` | Get document by number |

### Sanctions Screening (Open Sanctions)
| Tool | Description |
|------|-------------|
| `screen_entity` | Screen against 200+ global sanctions lists |
| `search_sanctions` | Search sanctions by keyword |
| `get_sanctions_record` | Get detailed entity record |

### Metadata
| Tool | Description |
|------|-------------|
| `list_supported_jurisdictions` | All jurisdictions with sources |
| `list_supported_sources` | Available data sources |
| `get_coverage_status` | Coverage gaps and planned additions |

## Response Schema

Every response includes:
```json
{
  "source": "courtlistener",
  "source_type": "case_law",
  "jurisdiction": "US",
  "title": "...",
  "citation": "928 F.3d 42",
  "source_url": "https://...",
  "retrieved_at": "2026-05-26T...",
  "not_legal_advice": true,
  "human_review_recommended": true
}
```

## Configuration

```json
{
  "mcpServers": {
    "legal": { "command": "mcp-legal" }
  }
}
```

## Governance

- All tools are `read_only` — no writes, no modifications
- Every response carries `not_legal_advice: true`
- Every response carries `human_review_recommended: true`
- Sanctions matches include `match_confidence` scores
- Governance gates: `no_legal_advice`, `citation_required`, `human_review_recommended`

## Roadmap

### v1.1 — Africa, Canada, Australia
- AfricanLII (Kenya, Nigeria, South Africa, Ghana, Uganda, Tanzania)
- CanLII (Canadian case law and legislation)
- Australian Federal Register of Legislation

### v1.2 — Contract Extraction
- `extract_contract_terms`, `extract_parties`, `extract_dates`
- `extract_obligations`, `extract_clauses`, `extract_governing_law`

## License

Apache-2.0
