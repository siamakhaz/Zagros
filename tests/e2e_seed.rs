use zagros::{CveDocument, db};

#[tokio::test]
#[ignore = "requires a running HelixDB"]
async fn seed_e2e_fixture() {
    let client = db::client().expect("HelixDB client");
    let doc = CveDocument {
        cve_id: "CVE-2099-0001".to_string(),
        title: "Zagros deterministic E2E fixture".to_string(),
        description: "Synthetic vulnerability record used only for CI smoke testing.".to_string(),
        published_at: None,
        updated_at: None,
    };
    db::upsert_document(&client, &doc)
        .await
        .expect("seed fixture into HelixDB");
}
