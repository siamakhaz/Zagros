use serde::Deserialize;
use std::fs;
use zagros::provenance::Provenance;
use zagros::sources::KnowledgeDoc;
use zagros::{CveDocument, rank_documents, rank_knowledge};

#[derive(Debug, Deserialize)]
struct Corpus {
    cves: Vec<CveFixture>,
    knowledge: Vec<KnowledgeFixture>,
}

#[derive(Debug, Deserialize)]
struct CveFixture {
    cve_id: String,
    title: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct KnowledgeFixture {
    id: String,
    source: String,
    name: String,
    description: String,
    tags: String,
}

#[derive(Debug, Deserialize)]
struct QueryCase {
    id: String,
    kind: String,
    query: String,
    expected_id: Option<String>,
}

fn load_fixture() -> (Vec<CveDocument>, Vec<KnowledgeDoc>, Vec<QueryCase>) {
    let root = env!("CARGO_MANIFEST_DIR");
    let corpus: Corpus = serde_json::from_str(
        &fs::read_to_string(format!("{root}/benchmarks/retrieval/corpus.json")).unwrap(),
    )
    .unwrap();
    let cases: Vec<QueryCase> = serde_json::from_str(
        &fs::read_to_string(format!("{root}/benchmarks/retrieval/queries.json")).unwrap(),
    )
    .unwrap();

    let cves = corpus
        .cves
        .into_iter()
        .map(|d| CveDocument {
            cve_id: d.cve_id,
            title: d.title,
            description: d.description,
            published_at: None,
            updated_at: None,
            provenance: Provenance::default(),
        })
        .collect();

    let knowledge = corpus
        .knowledge
        .into_iter()
        .map(|d| KnowledgeDoc {
            id: d.id,
            name: d.name,
            description: d.description,
            source: d.source,
            url: String::new(),
            tags: d.tags,
            provenance: Provenance::default(),
        })
        .collect();

    (cves, knowledge, cases)
}

#[test]
fn retrieval_quality_baseline() {
    let (cves, knowledge, cases) = load_fixture();

    let mut relevant = 0usize;
    let mut recall_at_1 = 0usize;
    let mut recall_at_5 = 0usize;
    let mut reciprocal_rank = 0.0f64;
    let mut rejection_cases = 0usize;
    let mut correct_rejections = 0usize;

    for case in &cases {
        let ids: Vec<String> = match case.kind.as_str() {
            "cve" => rank_documents(&cves, &case.query, 5)
                .into_iter()
                .map(|hit| hit.document.cve_id.clone())
                .collect(),
            "knowledge" => rank_knowledge(&knowledge, &case.query, 5)
                .into_iter()
                .map(|(doc, _)| doc.id.clone())
                .collect(),
            other => panic!("unknown benchmark kind {other:?} in {}", case.id),
        };

        match &case.expected_id {
            Some(expected) => {
                relevant += 1;
                if ids.first() == Some(expected) {
                    recall_at_1 += 1;
                }
                if let Some(position) = ids.iter().position(|id| id == expected) {
                    if position < 5 {
                        recall_at_5 += 1;
                    }
                    reciprocal_rank += 1.0 / (position as f64 + 1.0);
                }
            }
            None => {
                rejection_cases += 1;
                if ids.is_empty() {
                    correct_rejections += 1;
                }
            }
        }
    }

    let r1 = recall_at_1 as f64 / relevant as f64;
    let r5 = recall_at_5 as f64 / relevant as f64;
    let mrr = reciprocal_rank / relevant as f64;
    let rejection_rate = correct_rejections as f64 / rejection_cases as f64;

    println!(
        "retrieval benchmark: cases={}, recall@1={:.3}, recall@5={:.3}, mrr={:.3}, rejection={:.3}",
        cases.len(),
        r1,
        r5,
        mrr,
        rejection_rate
    );

    assert!(cases.len() >= 30, "benchmark must keep at least 30 cases");
    assert!(r1 >= 0.90, "Recall@1 regressed: {r1:.3}");
    assert!(r5 >= 0.98, "Recall@5 regressed: {r5:.3}");
    assert!(mrr >= 0.93, "MRR regressed: {mrr:.3}");
    assert!(
        rejection_rate >= 0.80,
        "irrelevant-query rejection regressed: {rejection_rate:.3}"
    );
}
