use crate::CveDocument;
/// HelixDB integration for the security RAG store.
///
/// Two node labels are used:
///
/// | Label       | Key field | What it holds                              |
/// |-------------|-----------|---------------------------------------------|
/// | `Cve`       | `cve_id`  | Official CVE records from CVEProject        |
/// | `Knowledge` | `doc_id`  | CWE, ASVS, CAPEC, ATT&CK, and future sources|
///
/// Both use the same delete-then-insert upsert strategy because HelixDB v3
/// does not expose a native upsert in the DSL.
///
/// Search loads all nodes of the requested label and ranks them in-process
/// with BM25 (`lib.rs`). HelixDB is the canonical persistence layer; the
/// old flat JSON file is no longer used.
use crate::provenance::{Provenance, legacy_provenance};
use crate::sources::KnowledgeDoc;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use helix_db::{Client, QueryRequest, dsl::prelude::*};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// URL of the HelixDB instance. Override with the `HELIX_URL` env var.
///
/// The Docker image (`ghcr.io/helixdb/helixdb`) binds its HTTP server to
/// `0.0.0.0:8080` inside the container (`HELIX_HTTP_ADDR`). The compose file
/// maps that to host port **47474** to avoid colliding with other services that
/// commonly use 8080. Set `HELIX_URL` explicitly if you use a different port.
pub fn helix_url() -> String {
    std::env::var("HELIX_URL").unwrap_or_else(|_| "http://localhost:47474".to_string())
}

/// Build a [`Client`] pointed at the local HelixDB instance.
pub fn client() -> Result<Client> {
    Client::new(Some(&helix_url())).context("failed to build HelixDB client")
}

// ---------------------------------------------------------------------------
// Write: upsert one document
// ---------------------------------------------------------------------------

/// Upsert a single [`CveDocument`] into HelixDB.
///
/// HelixDB has no native upsert so we:
/// 1. Delete any existing node with this `cve_id`.
/// 2. Insert the new node.
///
/// If nothing matches the delete it is a no-op — we ignore errors from it.
/// A record can be temporarily absent from the index between a failed upsert
/// and the next successful run.
pub async fn upsert_document(client: &Client, doc: &CveDocument) -> Result<()> {
    // --- delete existing node (ignore error; node may not exist yet) ---
    let delete_batch = write_batch()
        .var_as(
            "existing",
            g().n_with_label("Cve")
                .where_(Predicate::eq("cve_id", doc.cve_id.clone()))
                .drop(),
        )
        .returning([] as [&str; 0]);

    let _: sonic_rs::Value = client
        .query(QueryRequest::write(delete_batch))
        .send()
        .await
        .unwrap_or_default();

    // --- insert the new node ---
    let props: Vec<(&str, PropertyInput)> = vec![
        (
            "cve_id",
            PropertyInput::Value(PropertyValue::String(doc.cve_id.clone())),
        ),
        (
            "title",
            PropertyInput::Value(PropertyValue::String(doc.title.clone())),
        ),
        (
            "description",
            PropertyInput::Value(PropertyValue::String(doc.description.clone())),
        ),
        (
            "published_at",
            PropertyInput::Value(PropertyValue::String(
                doc.published_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            )),
        ),
        (
            "updated_at",
            PropertyInput::Value(PropertyValue::String(
                doc.updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            )),
        ),
        (
            "provenance_json",
            PropertyInput::Value(PropertyValue::String(
                serde_json::to_string(&doc.provenance).unwrap_or_default(),
            )),
        ),
    ];

    for attempt in 0..2 {
        let insert_batch = write_batch()
            .var_as(
                "node",
                g().add_n("Cve", props.clone())
                    .value_map(None::<Vec<String>>),
            )
            .returning(["node"]);

        match client
            .query::<sonic_rs::Value>(QueryRequest::write(insert_batch))
            .send()
            .await
        {
            Ok(_) => return Ok(()),
            Err(e) if attempt == 0 => {
                eprintln!("insert attempt 1 failed for {}: {e}; retrying", doc.cve_id);
            }
            Err(e) => return Err(e).with_context(|| "failed to insert CVE node into HelixDB"),
        }
    }
    unreachable!()
}

/// Batch-upsert a slice of documents, returning the count stored.
pub async fn upsert_documents(client: &Client, docs: &[CveDocument]) -> Result<usize> {
    for doc in docs {
        upsert_document(client, doc)
            .await
            .with_context(|| format!("failed to upsert {}", doc.cve_id))?;
    }
    Ok(docs.len())
}

// ---------------------------------------------------------------------------
// Read: load all documents
// ---------------------------------------------------------------------------

/// Raw HelixDB row shape for a `Cve` node value-map.
#[derive(Debug, Deserialize)]
struct CveRow {
    cve_id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    published_at: Option<String>,
    updated_at: Option<String>,
    provenance_json: Option<String>,
}

/// Wrapper that HelixDB returns for a `returning(["nodes"])` query.
#[derive(Debug, Deserialize)]
struct NodesResponse {
    nodes: Vec<CveRow>,
}

/// Load every CVE document stored in HelixDB.
pub async fn load_all(client: &Client) -> Result<Vec<CveDocument>> {
    let batch = read_batch()
        .var_as(
            "nodes",
            g().n_with_label("Cve").value_map(None::<Vec<String>>),
        )
        .returning(["nodes"]);

    let raw: NodesResponse = client
        .query(QueryRequest::read(batch))
        .send()
        .await
        .context("failed to load CVE nodes from HelixDB")?;

    Ok(raw.nodes.into_iter().filter_map(row_to_doc).collect())
}

/// Look up a single CVE by exact `cve_id`.
pub async fn get_by_id(client: &Client, cve_id: &str) -> Result<Option<CveDocument>> {
    let batch = read_batch()
        .var_as(
            "nodes",
            g().n_with_label("Cve")
                .where_(Predicate::eq("cve_id", cve_id.to_string()))
                .value_map(None::<Vec<String>>),
        )
        .returning(["nodes"]);

    let raw: NodesResponse = client
        .query(QueryRequest::read(batch))
        .send()
        .await
        .context("failed to look up CVE in HelixDB")?;

    Ok(raw.nodes.into_iter().filter_map(row_to_doc).next())
}

/// Return how many `Cve` nodes are stored.
pub async fn count(client: &Client) -> Result<usize> {
    Ok(load_all(client).await?.len())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn row_to_doc(row: CveRow) -> Option<CveDocument> {
    let cve_id = row.cve_id?;
    if cve_id.is_empty() {
        return None;
    }
    let title = row.title.unwrap_or_else(|| cve_id.clone());
    let description = row.description.unwrap_or_else(|| title.clone());
    let published_at = row
        .published_at
        .filter(|s| !s.is_empty())
        .and_then(|s| s.parse::<DateTime<Utc>>().ok());
    let updated_at = row
        .updated_at
        .filter(|s| !s.is_empty())
        .and_then(|s| s.parse::<DateTime<Utc>>().ok());

    let canonical_url = format!("https://www.cve.org/CVERecord?id={cve_id}");
    let provenance = row
        .provenance_json
        .as_deref()
        .and_then(|value| serde_json::from_str::<Provenance>(value).ok())
        .unwrap_or_else(|| legacy_provenance("cve", &canonical_url));

    Some(CveDocument {
        cve_id,
        title,
        description,
        published_at,
        updated_at,
        provenance,
    })
}

// ---------------------------------------------------------------------------
// Knowledge nodes  (CWE, ASVS, CAPEC, ATT&CK, …)
// ---------------------------------------------------------------------------

/// Upsert a single [`KnowledgeDoc`] as a `Knowledge` node in HelixDB.
///
/// A record can be temporarily absent from the index between a failed upsert
/// and the next successful run.
pub async fn upsert_knowledge(client: &Client, doc: &KnowledgeDoc) -> Result<()> {
    // Delete any existing node with this doc_id.
    let delete_batch = write_batch()
        .var_as(
            "existing",
            g().n_with_label("Knowledge")
                .where_(Predicate::eq("doc_id", doc.id.clone()))
                .drop(),
        )
        .returning([] as [&str; 0]);

    let _: sonic_rs::Value = client
        .query(QueryRequest::write(delete_batch))
        .send()
        .await
        .unwrap_or_default();

    // Insert the new node.
    let props: Vec<(&str, PropertyInput)> = vec![
        (
            "doc_id",
            PropertyInput::Value(PropertyValue::String(doc.id.clone())),
        ),
        (
            "name",
            PropertyInput::Value(PropertyValue::String(doc.name.clone())),
        ),
        (
            "description",
            PropertyInput::Value(PropertyValue::String(doc.description.clone())),
        ),
        (
            "source",
            PropertyInput::Value(PropertyValue::String(doc.source.clone())),
        ),
        (
            "url",
            PropertyInput::Value(PropertyValue::String(doc.url.clone())),
        ),
        (
            "tags",
            PropertyInput::Value(PropertyValue::String(doc.tags.clone())),
        ),
        (
            "provenance_json",
            PropertyInput::Value(PropertyValue::String(
                serde_json::to_string(&doc.provenance).unwrap_or_default(),
            )),
        ),
    ];

    for attempt in 0..2 {
        let insert_batch = write_batch()
            .var_as(
                "node",
                g().add_n("Knowledge", props.clone())
                    .value_map(None::<Vec<String>>),
            )
            .returning(["node"]);

        match client
            .query::<sonic_rs::Value>(QueryRequest::write(insert_batch))
            .send()
            .await
        {
            Ok(_) => return Ok(()),
            Err(e) if attempt == 0 => {
                eprintln!("insert attempt 1 failed for {}: {e}; retrying", doc.id);
            }
            Err(e) => {
                return Err(e)
                    .with_context(|| format!("failed to insert Knowledge node {}", doc.id));
            }
        }
    }
    unreachable!()
}

/// Batch-upsert a slice of knowledge documents.
pub async fn upsert_knowledge_batch(client: &Client, docs: &[KnowledgeDoc]) -> Result<usize> {
    for doc in docs {
        upsert_knowledge(client, doc)
            .await
            .with_context(|| format!("failed to upsert knowledge {}", doc.id))?;
    }
    Ok(docs.len())
}

/// Raw HelixDB row for a `Knowledge` node value-map.
#[derive(Debug, Deserialize)]
struct KnowledgeRow {
    doc_id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    source: Option<String>,
    url: Option<String>,
    tags: Option<String>,
    provenance_json: Option<String>,
}

#[derive(Debug, Deserialize)]
struct KnowledgeResponse {
    nodes: Vec<KnowledgeRow>,
}

/// Load all `Knowledge` nodes from HelixDB.
pub async fn load_all_knowledge(client: &Client) -> Result<Vec<KnowledgeDoc>> {
    let batch = read_batch()
        .var_as(
            "nodes",
            g().n_with_label("Knowledge").value_map(None::<Vec<String>>),
        )
        .returning(["nodes"]);

    let raw: KnowledgeResponse = client
        .query(QueryRequest::read(batch))
        .send()
        .await
        .context("failed to load Knowledge nodes from HelixDB")?;

    Ok(raw.nodes.into_iter().filter_map(row_to_knowledge).collect())
}

/// Count `Knowledge` nodes by source tag.
pub async fn count_knowledge_by_source(client: &Client) -> Result<Vec<(String, usize)>> {
    let all = load_all_knowledge(client).await?;
    let mut by_source: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for doc in &all {
        *by_source.entry(doc.source.clone()).or_default() += 1;
    }
    let mut counts: Vec<(String, usize)> = by_source.into_iter().collect();
    counts.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(counts)
}

fn row_to_knowledge(row: KnowledgeRow) -> Option<KnowledgeDoc> {
    let id = row.doc_id?;
    if id.is_empty() {
        return None;
    }
    let source = row.source.unwrap_or_default();
    let url = row.url.unwrap_or_default();
    let provenance = row
        .provenance_json
        .as_deref()
        .and_then(|value| serde_json::from_str::<Provenance>(value).ok())
        .unwrap_or_else(|| {
            legacy_provenance(
                if source.is_empty() {
                    "unknown"
                } else {
                    &source
                },
                &url,
            )
        });
    Some(KnowledgeDoc {
        id,
        name: row.name.unwrap_or_default(),
        description: row.description.unwrap_or_default(),
        source,
        url,
        tags: row.tags.unwrap_or_default(),
        provenance,
    })
}
