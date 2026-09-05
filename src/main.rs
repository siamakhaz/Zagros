use anyhow::Result;
use clap::{Parser, Subcommand, builder::TypedValueParser};
use cve_rag::{
    backfill_from_history, db, ingest_asvs, ingest_attack, ingest_capec, ingest_cwe, owned_hit,
    rank_documents, rank_knowledge, sync_cves_to_helix,
};
use std::{
    convert::TryFrom,
    io::{self, Write},
};

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Security RAG — CVE + CWE + ASVS + CAPEC + ATT&CK"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch latest changed CVEs and store them in HelixDB.
    Ingest {
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Walk deltaLog.json history and load up to N CVEs into HelixDB.
    Backfill {
        #[arg(long, default_value_t = 500, value_parser = clap::value_parser!(u64).range(1..=10000).try_map(|n| usize::try_from(n)))]
        limit: usize,
        #[arg(long, default_value_t = false)]
        verbose: bool,
    },
    /// Ingest a knowledge source into HelixDB.
    ///
    /// Available sources: cwe  asvs  capec  attack  all
    Source {
        /// Which source to load: cwe, asvs, capec, attack, all
        name: String,
    },
    /// Search CVE records loaded from HelixDB.
    Search {
        query: String,
        #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u64).range(1..=500).try_map(|n| usize::try_from(n)))]
        top_k: usize,
        #[arg(long)]
        json: bool,
    },
    /// Search the security knowledge base (CWE, ASVS, CAPEC, ATT&CK).
    Know {
        query: String,
        #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u64).range(1..=500).try_map(|n| usize::try_from(n)))]
        top_k: usize,
    },
    /// Open an interactive CVE search prompt.
    Interactive {
        #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u64).range(1..=500).try_map(|n| usize::try_from(n)))]
        top_k: usize,
    },
    /// Show HelixDB status: URL, CVE count, and knowledge source counts.
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Ingest { limit } => {
            let (changed, total) = sync_cves_to_helix(limit).await?;
            println!("ingested {changed} changed CVEs; {total} total in HelixDB");
        }
        Commands::Backfill { limit, verbose } => {
            println!("backfilling up to {limit} CVEs from deltaLog history…");
            let (fetched, total) = backfill_from_history(limit, verbose).await?;
            println!("fetched {fetched} CVEs; {total} total in HelixDB");
        }
        Commands::Source { name } => run_source(&name).await?,
        Commands::Search { query, top_k, json } => search_cve(&query, top_k, json).await?,
        Commands::Know { query, top_k } => search_knowledge(&query, top_k).await?,
        Commands::Interactive { top_k } => interactive(top_k).await?,
        Commands::Status => status().await?,
    }
    Ok(())
}

async fn run_source(name: &str) -> Result<()> {
    match name {
        "cwe" => {
            println!("ingesting MITRE CWE…");
            let (n, total) = ingest_cwe().await?;
            println!("CWE: {n} weaknesses stored; {total} total knowledge nodes");
        }
        "asvs" => {
            println!("ingesting OWASP ASVS 5.0…");
            let (n, total) = ingest_asvs().await?;
            println!("ASVS: {n} requirements stored; {total} total knowledge nodes");
        }
        "capec" => {
            println!("ingesting MITRE CAPEC…");
            let (n, total) = ingest_capec().await?;
            println!("CAPEC: {n} attack patterns stored; {total} total knowledge nodes");
        }
        "attack" => {
            println!("ingesting MITRE ATT&CK Enterprise…");
            let (n, total) = ingest_attack().await?;
            println!("ATT&CK: {n} techniques stored; {total} total knowledge nodes");
        }
        "all" => {
            for src in ["cwe", "asvs", "capec", "attack"] {
                Box::pin(run_source(src)).await?;
            }
        }
        other => anyhow::bail!("unknown source '{other}'; use: cwe asvs capec attack all"),
    }
    Ok(())
}

async fn search_cve(query: &str, top_k: usize, json: bool) -> Result<()> {
    let helix = db::client()?;
    let docs = db::load_all(&helix).await?;
    let hits = rank_documents(&docs, query, top_k);
    if json {
        let results: Vec<_> = hits.iter().map(owned_hit).collect();
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        for (i, hit) in hits.iter().enumerate() {
            let doc = hit.document;
            println!(
                "{}. {} | score {:.3} | updated {}",
                i + 1,
                doc.cve_id,
                hit.score,
                doc.updated_at
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "unknown".into())
            );
            println!("{}", doc.title);
            println!();
        }
        if hits.is_empty() {
            println!("No CVEs found.\n");
        }
    }
    Ok(())
}

async fn search_knowledge(query: &str, top_k: usize) -> Result<()> {
    let helix = db::client()?;
    let docs = db::load_all_knowledge(&helix).await?;
    let hits = rank_knowledge(&docs, query, top_k);
    if hits.is_empty() {
        println!("No knowledge results found.\n");
        return Ok(());
    }
    for (i, (doc, score)) in hits.iter().enumerate() {
        println!(
            "{}. {} [{}] | score {:.3}",
            i + 1,
            doc.id,
            doc.source.to_uppercase(),
            score
        );
        println!("   {}", doc.name);
        // Print first 200 chars of description
        let desc = &doc.description;
        let preview = if desc.len() > 200 { &desc[..200] } else { desc };
        println!("   {preview}…");
        if !doc.url.is_empty() {
            println!("   source: {}", doc.url);
        }
        println!();
    }
    Ok(())
}

async fn interactive(mut top_k: usize) -> Result<()> {
    let helix = db::client()?;
    let docs = db::load_all(&helix).await?;
    println!("CVE interactive search ({} records)", docs.len());
    println!("Commands: :help, :limit N, :quit");
    loop {
        print!("cve> ");
        io::stdout().flush()?;
        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            break;
        }
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        if matches!(input, ":quit" | ":q" | "quit" | "exit") {
            break;
        }
        if input == ":help" {
            println!("Enter a search phrase, CVE ID, product name, or vulnerability type.");
            println!(":limit N  — change result count; :quit — exit");
            continue;
        }
        if let Some(val) = input.strip_prefix(":limit ") {
            match val.trim().parse::<usize>() {
                Ok(n) if n > 0 && n <= 500 => {
                    top_k = n;
                    println!("limit set to {top_k}");
                }
                Ok(n) if n > 500 => {
                    eprintln!("limit must be between 1 and 500");
                }
                _ => println!("usage: :limit N  (N > 0)"),
            }
            continue;
        }
        let hits = rank_documents(&docs, input, top_k);
        if hits.is_empty() {
            println!("No results.\n");
        } else {
            for (i, hit) in hits.iter().enumerate() {
                println!("{}. {} | {:.3}", i + 1, hit.document.cve_id, hit.score);
                println!("   {}", hit.document.title);
            }
            println!();
        }
    }
    Ok(())
}

async fn status() -> Result<()> {
    let helix = db::client()?;
    let cve_count = db::count(&helix).await?;
    let by_source = db::count_knowledge_by_source(&helix).await?;
    println!("HelixDB URL  : {}", db::helix_url());
    println!("CVE nodes    : {cve_count}");
    if by_source.is_empty() {
        println!("Knowledge    : (none — run 'source all' to populate)");
    } else {
        for (src, n) in &by_source {
            println!("  {src:<12}: {n}");
        }
        let total: usize = by_source.iter().map(|(_, n)| n).sum();
        println!("  total       : {total}");
    }
    Ok(())
}
