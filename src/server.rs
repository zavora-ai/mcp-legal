use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use reqwest::Client;
use serde_json::{json, Value};

use crate::types::{JurisdictionInfo, LegalResult, SanctionsResult, SourceInfo};

fn now() -> String { chrono::Utc::now().to_rfc3339() }

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchQuery { pub query: String, pub limit: Option<u32> }

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CaseIdInput { pub case_id: String }

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LegislationInput { pub identifier: String }

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EntityInput { pub name: String }

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EmptyInput {}

#[derive(Clone)]
pub struct LegalServer { pub client: Client }

#[tool_router(server_handler)]
impl LegalServer {
    // === US Case Law (CourtListener) ===

    #[tool(description = "Search US case law opinions by keyword (CourtListener, 117M+ opinions). Returns case name, court, citations, date")]
    async fn search_cases(&self, Parameters(input): Parameters<SearchQuery>) -> String {
        let limit = input.limit.unwrap_or(5);
        let url = format!(
            "https://www.courtlistener.com/api/rest/v4/search/?q={}&type=o&page_size={}",
            input.query.replace(' ', "+"), limit
        );
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    let results: Vec<LegalResult> = data["results"].as_array().unwrap_or(&vec![]).iter().map(|r| {
                        LegalResult {
                            source: "courtlistener".into(),
                            source_type: "case_law".into(),
                            jurisdiction: "US".into(),
                            title: r["caseName"].as_str().unwrap_or_default().to_string(),
                            citation: r["citation"].as_array().and_then(|a| a.first()).and_then(|c| c.as_str()).map(String::from),
                            source_url: r["absolute_url"].as_str().map(|u| format!("https://www.courtlistener.com{u}")),
                            retrieved_at: now(),
                            published_at: r["dateFiled"].as_str().map(String::from),
                            effective_date: None,
                            version_status: "current".into(),
                            text: r["opinions"].as_array().and_then(|a| a.first()).and_then(|o| o["snippet"].as_str()).map(|s| s.chars().take(500).collect()),
                            summary: None,
                            metadata: json!({"court": r["court"], "docket_number": r["docketNumber"], "cite_count": r["citeCount"], "status": r["status"]}),
                            warnings: vec![],
                            not_legal_advice: true,
                            human_review_recommended: true,
                        }
                    }).collect();
                    serde_json::to_string_pretty(&results).unwrap_or_default()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get a specific US court opinion by cluster ID from CourtListener")]
    async fn get_case(&self, Parameters(input): Parameters<CaseIdInput>) -> String {
        let url = format!("https://www.courtlistener.com/api/rest/v4/search/?q=cluster_id:{}&type=o", input.case_id);
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    if let Some(r) = data["results"].as_array().and_then(|a| a.first()) {
                        let result = LegalResult {
                            source: "courtlistener".into(),
                            source_type: "case_law".into(),
                            jurisdiction: "US".into(),
                            title: r["caseName"].as_str().unwrap_or_default().to_string(),
                            citation: r["citation"].as_array().and_then(|a| a.first()).and_then(|c| c.as_str()).map(String::from),
                            source_url: r["absolute_url"].as_str().map(|u| format!("https://www.courtlistener.com{u}")),
                            retrieved_at: now(),
                            published_at: r["dateFiled"].as_str().map(String::from),
                            effective_date: None,
                            version_status: "current".into(),
                            text: r["opinions"].as_array().and_then(|a| a.first()).and_then(|o| o["snippet"].as_str()).map(String::from),
                            summary: None,
                            metadata: json!({"court": r["court"], "docket_number": r["docketNumber"], "judge": r["judge"]}),
                            warnings: vec![],
                            not_legal_advice: true,
                            human_review_recommended: true,
                        };
                        serde_json::to_string_pretty(&result).unwrap_or_default()
                    } else {
                        format!("No case found for ID {}", input.case_id)
                    }
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    // === UK Legislation ===

    #[tool(description = "Search UK legislation (Acts of Parliament, Statutory Instruments) by keyword")]
    async fn search_uk_legislation(&self, Parameters(input): Parameters<SearchQuery>) -> String {
        let limit = input.limit.unwrap_or(5);
        let url = format!("https://www.legislation.gov.uk/all?text={}", input.query.replace(' ', "+"));
        // UK Legislation returns Atom XML for search
        match self.client.get(&url).header("Accept", "application/atom+xml").send().await {
            Ok(resp) => match resp.text().await {
                Ok(xml) => parse_uk_legislation_search(&xml, limit as usize),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get a specific UK legislation document by identifier (e.g. ukpga/2018/12 for Data Protection Act 2018)")]
    async fn get_uk_legislation(&self, Parameters(input): Parameters<LegislationInput>) -> String {
        let url = format!("https://www.legislation.gov.uk/{}/contents", input.identifier);
        match self.client.get(&url).header("Accept", "application/xml").send().await {
            Ok(resp) => match resp.text().await {
                Ok(xml) => {
                    let title = extract_xml_tag(&xml, "dc:title").unwrap_or_default();
                    let result = LegalResult {
                        source: "uk_legislation".into(),
                        source_type: "legislation".into(),
                        jurisdiction: "UK".into(),
                        title,
                        citation: Some(input.identifier.clone()),
                        source_url: Some(format!("https://www.legislation.gov.uk/{}", input.identifier)),
                        retrieved_at: now(),
                        published_at: extract_xml_tag(&xml, "dc:date"),
                        effective_date: None,
                        version_status: "current".into(),
                        text: None,
                        summary: extract_xml_tag(&xml, "dc:description"),
                        metadata: json!({"identifier": input.identifier}),
                        warnings: vec![],
                        not_legal_advice: true,
                        human_review_recommended: true,
                    };
                    serde_json::to_string_pretty(&result).unwrap_or_default()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    // === EU Legislation (EUR-Lex) ===

    #[tool(description = "Search EU legislation, directives, and regulations on EUR-Lex by keyword")]
    async fn search_eu_legislation(&self, Parameters(input): Parameters<SearchQuery>) -> String {
        // EUR-Lex doesn't have a simple REST API; use the search page with structured output
        let url = format!(
            "https://eur-lex.europa.eu/search.html?scope=EURLEX&text={}&type=quick",
            input.query.replace(' ', "+")
        );
        let result = LegalResult {
            source: "eurlex".into(),
            source_type: "legislation".into(),
            jurisdiction: "EU".into(),
            title: format!("EUR-Lex search: {}", input.query),
            citation: None,
            source_url: Some(url),
            retrieved_at: now(),
            published_at: None,
            effective_date: None,
            version_status: "current".into(),
            text: None,
            summary: Some(format!("Search EUR-Lex for '{}'. Use the source_url to access results directly. For specific documents, use CELEX numbers (e.g. 32016R0679 for GDPR).", input.query)),
            metadata: json!({"note": "EUR-Lex does not provide a free JSON search API. Use CELEX identifiers for direct document access."}),
            warnings: vec!["EUR-Lex search requires browser access. Use get_eu_document with a CELEX number for direct retrieval.".into()],
            not_legal_advice: true,
            human_review_recommended: true,
        };
        serde_json::to_string_pretty(&result).unwrap_or_default()
    }

    #[tool(description = "Get an EU legal document by CELEX number (e.g. 32016R0679 for GDPR, 32000L0031 for E-Commerce Directive)")]
    async fn get_eu_document(&self, Parameters(input): Parameters<LegislationInput>) -> String {
        let url = format!("https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:{}", input.identifier);
        let result = LegalResult {
            source: "eurlex".into(),
            source_type: "legislation".into(),
            jurisdiction: "EU".into(),
            title: format!("CELEX:{}", input.identifier),
            citation: Some(format!("CELEX:{}", input.identifier)),
            source_url: Some(url),
            retrieved_at: now(),
            published_at: None,
            effective_date: None,
            version_status: "current".into(),
            text: None,
            summary: None,
            metadata: json!({"celex": input.identifier, "common_celex": {"GDPR": "32016R0679", "E-Commerce": "32000L0031", "AI_Act": "32024R1689", "DSA": "32022R2065", "DMA": "32022R1925"}}),
            warnings: vec![],
            not_legal_advice: true,
            human_review_recommended: true,
        };
        serde_json::to_string_pretty(&result).unwrap_or_default()
    }

    // === US Regulations (Federal Register) ===

    #[tool(description = "Search the US Federal Register for regulations, proposed rules, and agency notices")]
    async fn search_federal_register(&self, Parameters(input): Parameters<SearchQuery>) -> String {
        let limit = input.limit.unwrap_or(5);
        let url = format!(
            "https://www.federalregister.gov/api/v1/documents.json?conditions%5Bterm%5D={}&per_page={}",
            input.query.replace(' ', "+"), limit
        );
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    let results: Vec<LegalResult> = data["results"].as_array().unwrap_or(&vec![]).iter().map(|r| {
                        LegalResult {
                            source: "federal_register".into(),
                            source_type: "regulation".into(),
                            jurisdiction: "US".into(),
                            title: r["title"].as_str().unwrap_or_default().to_string(),
                            citation: r["citation"].as_str().map(String::from),
                            source_url: r["html_url"].as_str().map(String::from),
                            retrieved_at: now(),
                            published_at: r["publication_date"].as_str().map(String::from),
                            effective_date: r["effective_on"].as_str().map(String::from),
                            version_status: "current".into(),
                            text: r["abstract"].as_str().map(String::from),
                            summary: None,
                            metadata: json!({
                                "document_type": r["type"],
                                "agencies": r["agencies"].as_array().map(|a| a.iter().filter_map(|ag| ag["name"].as_str()).collect::<Vec<_>>()),
                                "document_number": r["document_number"]
                            }),
                            warnings: vec![],
                            not_legal_advice: true,
                            human_review_recommended: true,
                        }
                    }).collect();
                    serde_json::to_string_pretty(&results).unwrap_or_default()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get a specific Federal Register document by document number")]
    async fn get_federal_register_document(&self, Parameters(input): Parameters<LegislationInput>) -> String {
        let url = format!("https://www.federalregister.gov/api/v1/documents/{}.json", input.identifier);
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(r) => {
                    let result = LegalResult {
                        source: "federal_register".into(),
                        source_type: "regulation".into(),
                        jurisdiction: "US".into(),
                        title: r["title"].as_str().unwrap_or_default().to_string(),
                        citation: r["citation"].as_str().map(String::from),
                        source_url: r["html_url"].as_str().map(String::from),
                        retrieved_at: now(),
                        published_at: r["publication_date"].as_str().map(String::from),
                        effective_date: r["effective_on"].as_str().map(String::from),
                        version_status: "current".into(),
                        text: r["abstract"].as_str().map(String::from),
                        summary: r["action"].as_str().map(String::from),
                        metadata: json!({"type": r["type"], "agencies": r["agencies"], "docket_ids": r["docket_ids"]}),
                        warnings: vec![],
                        not_legal_advice: true,
                        human_review_recommended: true,
                    };
                    serde_json::to_string_pretty(&result).unwrap_or_default()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    // === Australia (Federal Register of Legislation) ===

    #[tool(description = "Search Australian legislation (Acts, Legislative Instruments) by keyword")]
    async fn search_australian_legislation(&self, Parameters(input): Parameters<SearchQuery>) -> String {
        let limit = input.limit.unwrap_or(5);
        let url = format!("https://api.prod.legislation.gov.au/v1/titles?$filter=contains(name,'{}')&$top={}", input.query.replace(' ', "%20"), limit);
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    let results: Vec<LegalResult> = data["value"].as_array().unwrap_or(&vec![]).iter().map(|r| {
                        LegalResult {
                            source: "australian_frl".into(), source_type: "legislation".into(), jurisdiction: "AU".into(),
                            title: r["name"].as_str().unwrap_or_default().to_string(),
                            citation: r["id"].as_str().map(String::from),
                            source_url: r["id"].as_str().map(|id| format!("https://www.legislation.gov.au/{}/latest", id)),
                            retrieved_at: now(),
                            published_at: r["makingDate"].as_str().map(|d| d.chars().take(10).collect()),
                            effective_date: None,
                            version_status: if r["isInForce"].as_bool().unwrap_or(false) { "current" } else { "not_in_force" }.into(),
                            text: None, summary: None,
                            metadata: json!({"collection": r["collection"], "sub_collection": r["subCollection"]}),
                            warnings: vec![], not_legal_advice: true, human_review_recommended: true,
                        }
                    }).collect();
                    serde_json::to_string_pretty(&results).unwrap_or_default()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    // === Japan (e-Gov Laws API) ===

    #[tool(description = "Search Japanese laws via e-Gov API (Constitution, Acts, Cabinet Orders). Note: law names are in Japanese")]
    async fn search_japanese_laws(&self, Parameters(input): Parameters<SearchQuery>) -> String {
        let url = "https://elaws.e-gov.go.jp/api/1/lawlists/1";
        match self.client.get(url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(xml) => {
                    let q = input.query.to_lowercase();
                    let limit = input.limit.unwrap_or(5) as usize;
                    let mut results = Vec::new();
                    let mut rest = xml.as_str();
                    while let Some(start) = rest.find("<LawNameListInfo>") {
                        let end = match rest[start..].find("</LawNameListInfo>") { Some(i) => start + i + 18, None => break };
                        let entry = &rest[start..end];
                        let law_name = extract_xml_tag(entry, "LawName").unwrap_or_default();
                        let law_id = extract_xml_tag(entry, "LawId").unwrap_or_default();
                        if law_name.to_lowercase().contains(&q) || law_id.to_lowercase().contains(&q) {
                            results.push(LegalResult {
                                source: "japan_egov".into(), source_type: "legislation".into(), jurisdiction: "JP".into(),
                                title: law_name, citation: Some(law_id.clone()),
                                source_url: Some(format!("https://elaws.e-gov.go.jp/document?lawid={}", law_id)),
                                retrieved_at: now(), published_at: None, effective_date: None,
                                version_status: "current".into(), text: None, summary: None,
                                metadata: json!({"law_id": law_id}),
                                warnings: vec!["Japanese law text is authoritative in Japanese only.".into()],
                                not_legal_advice: true, human_review_recommended: true,
                            });
                        }
                        if results.len() >= limit { break; }
                        rest = &rest[end..];
                    }
                    if results.is_empty() {
                        json!({"note": "No matches. Japan e-Gov returns Japanese law names. Try Japanese keywords or law IDs.", "not_legal_advice": true}).to_string()
                    } else { serde_json::to_string_pretty(&results).unwrap_or_default() }
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    // === Sanctions (Open Sanctions) ===

    #[tool(description = "Screen an entity against global sanctions lists (200+ lists, PEPs, watchlists). Returns match confidence")]
    async fn screen_entity(&self, Parameters(input): Parameters<EntityInput>) -> String {
        let url = format!("https://api.opensanctions.org/search/default?q={}&limit=5", input.name.replace(' ', "+"));
        match self.client.get(&url).header("User-Agent", "mcp-legal/1.0").send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    let results: Vec<SanctionsResult> = data["results"].as_array().unwrap_or(&vec![]).iter().map(|r| {
                        SanctionsResult {
                            source: "opensanctions".into(),
                            entity_name: r["caption"].as_str().unwrap_or_default().to_string(),
                            match_confidence: r["score"].as_f64(),
                            matched_fields: r["properties"].as_object().map(|p| p.keys().cloned().collect()).unwrap_or_default(),
                            aliases: r["properties"]["alias"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
                            list_authority: None,
                            datasets: r["datasets"].as_array().map(|a| a.iter().filter_map(|d| d["name"].as_str().map(String::from)).collect()).unwrap_or_default(),
                            date_listed: r["first_seen"].as_str().map(String::from),
                            source_url: r["id"].as_str().map(|id| format!("https://opensanctions.org/entities/{id}/")),
                            retrieved_at: now(),
                            not_legal_advice: true,
                            human_review_recommended: true,
                        }
                    }).collect();
                    if results.is_empty() {
                        json!({"query": input.name, "matches": 0, "status": "no_match", "not_legal_advice": true}).to_string()
                    } else {
                        serde_json::to_string_pretty(&results).unwrap_or_default()
                    }
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Search sanctions lists by keyword (companies, individuals, vessels, organizations)")]
    async fn search_sanctions(&self, Parameters(input): Parameters<SearchQuery>) -> String {
        self.screen_entity(Parameters(EntityInput { name: input.query })).await
    }

    #[tool(description = "Get detailed sanctions record by entity ID from Open Sanctions")]
    async fn get_sanctions_record(&self, Parameters(input): Parameters<CaseIdInput>) -> String {
        let url = format!("https://api.opensanctions.org/entities/{}", input.case_id);
        match self.client.get(&url).header("User-Agent", "mcp-legal/1.0").send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => serde_json::to_string_pretty(&data).unwrap_or_default(),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    // === Sweden (Riksdagen JSON API) ===

    #[tool(description = "Search Swedish legislation via Riksdagen API (returns structured results with dates and IDs)")]
    async fn search_swedish_legislation(&self, Parameters(input): Parameters<SearchQuery>) -> String {
        let limit = input.limit.unwrap_or(5);
        let url = format!("https://data.riksdagen.se/dokumentlista/?sok={}&doktyp=sfs&utformat=json&antal={}", input.query.replace(' ', "+"), limit);
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    let docs = data["dokumentlista"]["dokument"].as_array().unwrap_or(&vec![]).clone();
                    let results: Vec<LegalResult> = docs.iter().map(|d| LegalResult {
                        source: "riksdagen".into(), source_type: "legislation".into(), jurisdiction: "SE".into(),
                        title: d["titel"].as_str().unwrap_or_default().to_string(),
                        citation: d["id"].as_str().map(String::from),
                        source_url: d["dokument_url_html"].as_str().map(|u| format!("https:{u}")),
                        retrieved_at: now(), published_at: d["datum"].as_str().map(String::from),
                        effective_date: None, version_status: "current".into(),
                        text: None, summary: d["summary"].as_str().map(String::from),
                        metadata: json!({"id": d["id"], "rm": d["rm"]}),
                        warnings: vec!["Swedish text. Use translation tools for English.".into()],
                        not_legal_advice: true, human_review_recommended: true,
                    }).collect();
                    serde_json::to_string_pretty(&results).unwrap_or_default()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    // === Norway (Lovdata) ===

    #[tool(description = "Get Norwegian law in English by Lovdata path (e.g. lov/2018-06-15-38 for Personal Data Act)")]
    async fn get_norwegian_law(&self, Parameters(input): Parameters<LegislationInput>) -> String {
        let url = format!("https://lovdata.no/dokument/NLE/{}", input.identifier);
        let result = LegalResult {
            source: "lovdata".into(), source_type: "legislation".into(), jurisdiction: "NO".into(),
            title: format!("Norwegian Law: {}", input.identifier),
            citation: Some(input.identifier.clone()),
            source_url: Some(url),
            retrieved_at: now(), published_at: None, effective_date: None,
            version_status: "current".into(), text: None, summary: None,
            metadata: json!({"identifier": input.identifier, "common_laws": {"lov/2018-06-15-38": "Personal Data Act", "lov/2005-06-17-62": "Working Environment Act", "lov/1997-06-13-44": "Companies Act"}}),
            warnings: vec![], not_legal_advice: true, human_review_recommended: true,
        };
        serde_json::to_string_pretty(&result).unwrap_or_default()
    }

    // === South Korea (KLRI) ===

    #[tool(description = "Get South Korean law in English by KLRI hseq number (e.g. 53044 for Personal Information Protection Act)")]
    async fn get_korean_law(&self, Parameters(input): Parameters<LegislationInput>) -> String {
        let url = format!("https://elaw.klri.re.kr/eng_service/lawView.do?hseq={}&lang=ENG", input.identifier);
        match self.client.get(&url).send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let title = if status == 200 {
                    resp.text().await.ok().and_then(|h| extract_html_title(&h)).unwrap_or_else(|| format!("Korean Law hseq:{}", input.identifier))
                } else { format!("Korean Law hseq:{}", input.identifier) };
                let result = LegalResult {
                    source: "klri".into(), source_type: "legislation".into(), jurisdiction: "KR".into(),
                    title, citation: Some(input.identifier.clone()),
                    source_url: Some(url),
                    retrieved_at: now(), published_at: None, effective_date: None,
                    version_status: if status == 200 { "current" } else { "unknown" }.into(),
                    text: None, summary: None,
                    metadata: json!({"hseq": input.identifier, "common_laws": {"53044": "Personal Information Protection Act", "46795": "Act on Promotion of IT Network Use", "55211": "Credit Information Act"}}),
                    warnings: vec![], not_legal_advice: true, human_review_recommended: true,
                };
                serde_json::to_string_pretty(&result).unwrap_or_default()
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    // === Germany ===

    #[tool(description = "Get German federal law in English translation (e.g. bdsg for Data Protection, bgb for Civil Code, stgb for Criminal Code)")]
    async fn get_german_law(&self, Parameters(input): Parameters<LegislationInput>) -> String {
        let code = input.identifier.to_lowercase();
        let url = format!("https://www.gesetze-im-internet.de/englisch_{}/", code);
        match self.client.get(&url).send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let title = if status == 200 {
                    match resp.text().await {
                        Ok(html) => extract_html_title(&html).unwrap_or_else(|| format!("German Law: {}", code)),
                        Err(_) => format!("German Law: {}", code),
                    }
                } else { format!("German Law: {}", code) };
                let result = LegalResult {
                    source: "gesetze_im_internet".into(), source_type: "legislation".into(), jurisdiction: "DE".into(),
                    title, citation: Some(code.to_uppercase()),
                    source_url: Some(url),
                    retrieved_at: now(), published_at: None, effective_date: None,
                    version_status: if status == 200 { "current" } else { "unknown" }.into(),
                    text: None, summary: None,
                    metadata: json!({"code": code, "language": "english", "common_codes": {"BDSG": "Data Protection", "BGB": "Civil Code", "StGB": "Criminal Code", "HGB": "Commercial Code", "GG": "Basic Law (Constitution)", "AktG": "Stock Corporation Act", "GmbHG": "Limited Liability Companies Act", "UrhG": "Copyright Act"}}),
                    warnings: vec!["English translation provided for convenience. German text is authoritative.".into()],
                    not_legal_advice: true, human_review_recommended: true,
                };
                serde_json::to_string_pretty(&result).unwrap_or_default()
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get Swiss federal law by ELI identifier (e.g. cc/2022/491 for Data Protection Act, cc/1907/637 for Civil Code)")]
    async fn get_swiss_law(&self, Parameters(input): Parameters<LegislationInput>) -> String {
        let url = format!("https://www.fedlex.admin.ch/eli/{}/en", input.identifier);
        let result = LegalResult {
            source: "fedlex".into(), source_type: "legislation".into(), jurisdiction: "CH".into(),
            title: format!("Swiss Law: {}", input.identifier),
            citation: Some(input.identifier.clone()),
            source_url: Some(url),
            retrieved_at: now(), published_at: None, effective_date: None,
            version_status: "current".into(),
            text: None, summary: None,
            metadata: json!({"eli": input.identifier, "common_laws": {"cc/2022/491": "Data Protection Act (nDSG)", "cc/1907/637": "Civil Code (ZGB)", "cc/1911/321": "Code of Obligations (OR)", "cc/1937/399": "Criminal Code (StGB)"}}),
            warnings: vec![], not_legal_advice: true, human_review_recommended: true,
        };
        serde_json::to_string_pretty(&result).unwrap_or_default()
    }

    // === Canada (Justice Laws) ===

    #[tool(description = "Get Canadian federal legislation full text by act code (e.g. P-21 for Privacy Act, C-46 for Criminal Code, A-1 for Access to Information)")]
    async fn get_canadian_law(&self, Parameters(input): Parameters<LegislationInput>) -> String {
        let url = format!("https://laws-lois.justice.gc.ca/eng/XML/{}.xml", input.identifier);
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(xml) => {
                    let title = extract_xml_tag(&xml, "ShortTitle").or_else(|| extract_xml_tag(&xml, "LongTitle")).unwrap_or_default();
                    let text_excerpt = xml.chars().take(3000).collect::<String>();
                    let result = LegalResult {
                        source: "canada_justice_laws".into(), source_type: "legislation".into(), jurisdiction: "CA".into(),
                        title, citation: Some(input.identifier.clone()),
                        source_url: Some(format!("https://laws-lois.justice.gc.ca/eng/acts/{}/", input.identifier)),
                        retrieved_at: now(), published_at: None, effective_date: None,
                        version_status: "current".into(),
                        text: Some(text_excerpt),
                        summary: extract_xml_tag(&xml, "LongTitle"),
                        metadata: json!({"act_code": input.identifier, "format": "xml_full_text"}),
                        warnings: vec![], not_legal_advice: true, human_review_recommended: true,
                    };
                    serde_json::to_string_pretty(&result).unwrap_or_default()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    // === Ireland (Irish Statute Book) ===

    #[tool(description = "Get Irish legislation by year and act number (e.g. 2018/act/7 for Data Protection Act 2018)")]
    async fn get_irish_legislation(&self, Parameters(input): Parameters<LegislationInput>) -> String {
        let url = format!("https://www.irishstatutebook.ie/eli/{}/enacted/en/html", input.identifier);
        match self.client.get(&url).send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                match resp.text().await {
                    Ok(html) => {
                        let title = extract_html_title(&html).unwrap_or_default();
                        let result = LegalResult {
                            source: "irish_statute_book".into(), source_type: "legislation".into(), jurisdiction: "IE".into(),
                            title, citation: Some(input.identifier.clone()),
                            source_url: Some(url),
                            retrieved_at: now(), published_at: None, effective_date: None,
                            version_status: if status == 200 { "current" } else { "unknown" }.into(),
                            text: None, summary: None,
                            metadata: json!({"identifier": input.identifier, "format": "html"}),
                            warnings: vec![], not_legal_advice: true, human_review_recommended: true,
                        };
                        serde_json::to_string_pretty(&result).unwrap_or_default()
                    }
                    Err(e) => format!("Error: {e}"),
                }
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    // === New Zealand ===

    #[tool(description = "Get New Zealand legislation by identifier (e.g. act/public/2020/0031 for Privacy Act 2020)")]
    async fn get_nz_legislation(&self, Parameters(input): Parameters<LegislationInput>) -> String {
        let url = format!("https://legislation.govt.nz/{}/latest/whole.html", input.identifier);
        match self.client.get(&url).send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let result = LegalResult {
                    source: "nz_legislation".into(), source_type: "legislation".into(), jurisdiction: "NZ".into(),
                    title: format!("NZ {}", input.identifier),
                    citation: Some(input.identifier.clone()),
                    source_url: Some(url),
                    retrieved_at: now(), published_at: None, effective_date: None,
                    version_status: if status == 200 { "current" } else { "unknown" }.into(),
                    text: None, summary: None,
                    metadata: json!({"identifier": input.identifier, "common_ids": {"Privacy_Act_2020": "act/public/2020/0031", "Employment_Relations": "act/public/2000/0024", "Companies_Act": "act/public/1993/0105"}}),
                    warnings: vec![], not_legal_advice: true, human_review_recommended: true,
                };
                serde_json::to_string_pretty(&result).unwrap_or_default()
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    // === Metadata ===

    #[tool(description = "List all supported jurisdictions and their available legal data sources")]
    async fn list_supported_jurisdictions(&self, Parameters(_input): Parameters<EmptyInput>) -> String {
        let jurisdictions = vec![
            JurisdictionInfo { jurisdiction: "US".into(), sources: vec![
                SourceInfo { name: "CourtListener".into(), data_types: vec!["case_law".into(), "opinions".into(), "dockets".into()], update_cadence: "daily".into(), reliability: "high".into() },
                SourceInfo { name: "Federal Register".into(), data_types: vec!["regulations".into(), "proposed_rules".into(), "notices".into()], update_cadence: "daily".into(), reliability: "high".into() },
            ]},
            JurisdictionInfo { jurisdiction: "UK".into(), sources: vec![
                SourceInfo { name: "legislation.gov.uk".into(), data_types: vec!["acts".into(), "statutory_instruments".into()], update_cadence: "daily".into(), reliability: "high".into() },
            ]},
            JurisdictionInfo { jurisdiction: "EU".into(), sources: vec![
                SourceInfo { name: "EUR-Lex".into(), data_types: vec!["regulations".into(), "directives".into(), "decisions".into()], update_cadence: "daily".into(), reliability: "high".into() },
            ]},
            JurisdictionInfo { jurisdiction: "Global".into(), sources: vec![
                SourceInfo { name: "Open Sanctions".into(), data_types: vec!["sanctions".into(), "peps".into(), "watchlists".into()], update_cadence: "daily".into(), reliability: "high".into() },
            ]},
            JurisdictionInfo { jurisdiction: "CA".into(), sources: vec![
                SourceInfo { name: "Justice Laws (XML)".into(), data_types: vec!["federal_acts".into(), "regulations".into()], update_cadence: "weekly".into(), reliability: "high".into() },
            ]},
            JurisdictionInfo { jurisdiction: "IE".into(), sources: vec![
                SourceInfo { name: "Irish Statute Book".into(), data_types: vec!["acts".into()], update_cadence: "as_enacted".into(), reliability: "high".into() },
            ]},
            JurisdictionInfo { jurisdiction: "NZ".into(), sources: vec![
                SourceInfo { name: "NZ Legislation".into(), data_types: vec!["acts".into(), "regulations".into()], update_cadence: "weekly".into(), reliability: "high".into() },
            ]},
        ];
        serde_json::to_string_pretty(&jurisdictions).unwrap_or_default()
    }

    #[tool(description = "List all available legal data sources with their capabilities")]
    async fn list_supported_sources(&self, Parameters(_input): Parameters<EmptyInput>) -> String {
        let sources = json!([
            {"name": "CourtListener", "jurisdiction": "US", "types": ["case_law"], "api": "REST v4", "auth": "none for search", "url": "https://www.courtlistener.com"},
            {"name": "Federal Register", "jurisdiction": "US", "types": ["regulations"], "api": "REST", "auth": "none", "url": "https://www.federalregister.gov"},
            {"name": "UK Legislation", "jurisdiction": "UK", "types": ["legislation"], "api": "REST/XML", "auth": "none", "url": "https://www.legislation.gov.uk"},
            {"name": "EUR-Lex", "jurisdiction": "EU", "types": ["legislation", "directives"], "api": "CELEX lookup", "auth": "none", "url": "https://eur-lex.europa.eu"},
            {"name": "Open Sanctions", "jurisdiction": "Global", "types": ["sanctions", "peps"], "api": "REST", "auth": "none", "url": "https://opensanctions.org"},
        ]);
        serde_json::to_string_pretty(&sources).unwrap_or_default()
    }

    #[tool(description = "Get coverage status and known gaps for a jurisdiction")]
    async fn get_coverage_status(&self, Parameters(_input): Parameters<EmptyInput>) -> String {
        let status = json!({
            "covered": {
                "US": {"case_law": "full (CourtListener)", "regulations": "full (Federal Register)", "legislation": "partial (via CourtListener citations)"},
                "UK": {"legislation": "full (legislation.gov.uk)", "case_law": "not yet (Phase 2)"},
                "EU": {"legislation": "partial (CELEX lookup)", "case_law": "not yet (HUDOC Phase 2)"},
                "Global": {"sanctions": "full (Open Sanctions, 200+ lists)"}
            },
            "planned_phase_2": ["AfricanLII (Kenya, Nigeria, SA)", "CanLII (Canada)", "Australian FRL", "HUDOC (ECHR)", "India (Indian Kanoon)"],
            "not_legal_advice": true
        });
        serde_json::to_string_pretty(&status).unwrap_or_default()
    }
}

// --- Helpers ---

fn parse_uk_legislation_search(xml: &str, limit: usize) -> String {
    let mut results = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<entry") {
        let end = match rest[start..].find("</entry>") {
            Some(i) => start + i + 8,
            None => break,
        };
        let entry = &rest[start..end];
        let title = extract_xml_tag(entry, "title").unwrap_or_default();
        let link = entry.find("href=\"").and_then(|i| {
            let s = &entry[i + 6..];
            s.find('"').map(|e| s[..e].to_string())
        });
        let updated = extract_xml_tag(entry, "updated");
        if !title.is_empty() {
            results.push(LegalResult {
                source: "uk_legislation".into(),
                source_type: "legislation".into(),
                jurisdiction: "UK".into(),
                title,
                citation: None,
                source_url: link,
                retrieved_at: now(),
                published_at: updated,
                effective_date: None,
                version_status: "current".into(),
                text: None,
                summary: extract_xml_tag(entry, "summary"),
                metadata: json!({}),
                warnings: vec![],
                not_legal_advice: true,
                human_review_recommended: true,
            });
        }
        if results.len() >= limit { break; }
        rest = &rest[end..];
    }
    if results.is_empty() {
        json!({"note": "UK Legislation search returned no structured results. Try a specific identifier with get_uk_legislation (e.g. ukpga/2018/12).", "not_legal_advice": true}).to_string()
    } else {
        serde_json::to_string_pretty(&results).unwrap_or_default()
    }
}

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)?;
    let content_start = xml[start..].find('>')? + start + 1;
    let end = xml[content_start..].find(&close)?;
    let raw = &xml[content_start..content_start + end];
    Some(raw.trim().to_string())
}

fn extract_html_title(html: &str) -> Option<String> {
    let start = html.find("<title>")? + 7;
    let end = html[start..].find("</title>")? + start;
    Some(html[start..end].trim().to_string())
}
