use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::SocketAddr, sync::Arc};
use zagros::{CveDocument, db, rank_documents, rank_knowledge};

#[derive(Clone)]
struct AppState {
    helix_url: String,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    connected: bool,
    cves: usize,
    knowledge: usize,
    total: usize,
    sources: BTreeMap<String, usize>,
    issues: usize,
}

#[derive(Debug, Serialize, Clone)]
struct CveView {
    cve_id: String,
    title: String,
    description: String,
    published_at: String,
    updated_at: String,
    source_url: String,
}

#[derive(Debug, Serialize, Clone)]
struct KnowledgeView {
    id: String,
    name: String,
    description: String,
    source: String,
    url: String,
    tags: String,
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: Option<String>,
    kind: Option<String>,
    top_k: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ReviewQuery {
    dataset: Option<String>,
    source: Option<String>,
    q: Option<String>,
    quality: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Serialize, Clone)]
struct ReviewRow {
    kind: String,
    id: String,
    name: String,
    source: String,
    summary: String,
    metadata: String,
    issues: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ReviewPage {
    page: usize,
    page_size: usize,
    total: usize,
    pages: usize,
    items: Vec<ReviewRow>,
}

#[derive(Debug, Serialize)]
struct SearchResponse {
    query: String,
    kind: String,
    count: usize,
    results: Vec<SearchRow>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "item")]
enum SearchRow {
    Cve { score: f64, item: CveView },
    Knowledge { score: f64, item: KnowledgeView },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = std::env::var("UI_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8788);
    let bind_addr: SocketAddr = format!("0.0.0.0:{port}").parse()?;
    let state = Arc::new(AppState {
        helix_url: std::env::var("HELIX_URL")
            .unwrap_or_else(|_| "http://host.docker.internal:47474".to_string()),
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/api/status", get(status))
        .route("/api/cves", get(cves))
        .route("/api/cves/{cve_id}", get(cve))
        .route("/api/knowledge", get(knowledge))
        .route("/api/knowledge/{doc_id}", get(knowledge_doc))
        .route("/api/review", get(review))
        .route("/api/search", get(search))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    println!("Zagros UI listening on http://{bind_addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn status(State(state): State<Arc<AppState>>) -> axum::response::Response {
    let client = match db::client() {
        Ok(client) => client,
        Err(err) => return api_error(StatusCode::BAD_GATEWAY, err),
    };
    let cves = match db::load_all(&client).await {
        Ok(docs) => docs,
        Err(err) => return api_error(StatusCode::BAD_GATEWAY, err),
    };
    let knowledge = match db::load_all_knowledge(&client).await {
        Ok(docs) => docs,
        Err(err) => return api_error(StatusCode::BAD_GATEWAY, err),
    };

    Json(StatusResponse {
        connected: !state.helix_url.is_empty(),
        cves: cves.len(),
        knowledge: knowledge.len(),
        total: cves.len() + knowledge.len(),
        sources: knowledge.iter().fold(BTreeMap::new(), |mut counts, doc| {
            *counts.entry(doc.source.clone()).or_default() += 1;
            counts
        }),
        issues: cves
            .iter()
            .map(cve_issues)
            .filter(|v| !v.is_empty())
            .count()
            + knowledge
                .iter()
                .map(knowledge_issues)
                .filter(|v| !v.is_empty())
                .count(),
    })
    .into_response()
}

async fn review(Query(query): Query<ReviewQuery>) -> axum::response::Response {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(50).clamp(10, 100);
    let dataset = query.dataset.as_deref().unwrap_or("all");
    let source = query.source.as_deref().unwrap_or("all");
    let quality = query.quality.as_deref().unwrap_or("all");
    let term = query.q.unwrap_or_default().trim().to_lowercase();
    let client = match db::client() {
        Ok(client) => client,
        Err(err) => return api_error(StatusCode::BAD_GATEWAY, err),
    };

    let mut rows = Vec::new();
    if dataset == "all" || dataset == "cve" {
        let docs = match db::load_all(&client).await {
            Ok(docs) => docs,
            Err(err) => return api_error(StatusCode::BAD_GATEWAY, err),
        };
        rows.extend(docs.iter().map(|doc| {
            ReviewRow {
                kind: "cve".into(),
                id: doc.cve_id.clone(),
                name: doc.title.clone(),
                source: "cve.org".into(),
                summary: doc.description.clone(),
                metadata: doc
                    .updated_at
                    .or(doc.published_at)
                    .map(|d| d.to_rfc3339())
                    .unwrap_or_default(),
                issues: cve_issues(doc),
            }
        }));
    }
    if dataset == "all" || dataset == "knowledge" {
        let docs = match db::load_all_knowledge(&client).await {
            Ok(docs) => docs,
            Err(err) => return api_error(StatusCode::BAD_GATEWAY, err),
        };
        rows.extend(
            docs.iter()
                .filter(|doc| source == "all" || doc.source == source)
                .map(|doc| ReviewRow {
                    kind: "knowledge".into(),
                    id: doc.id.clone(),
                    name: doc.name.clone(),
                    source: doc.source.clone(),
                    summary: doc.description.clone(),
                    metadata: doc.tags.clone(),
                    issues: knowledge_issues(doc),
                }),
        );
    }
    rows.retain(|row| {
        let text_match = term.is_empty()
            || row.id.to_lowercase().contains(&term)
            || row.name.to_lowercase().contains(&term)
            || row.summary.to_lowercase().contains(&term)
            || row.metadata.to_lowercase().contains(&term);
        let quality_match = quality != "issues" || !row.issues.is_empty();
        text_match && quality_match
    });
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    let total = rows.len();
    let pages = total.div_ceil(page_size).max(1);
    let safe_page = page.min(pages);
    let start = (safe_page - 1) * page_size;
    let items = rows.into_iter().skip(start).take(page_size).collect();
    Json(ReviewPage {
        page: safe_page,
        page_size,
        total,
        pages,
        items,
    })
    .into_response()
}

async fn cves() -> axum::response::Response {
    let client = match db::client() {
        Ok(client) => client,
        Err(err) => return api_error(StatusCode::BAD_GATEWAY, err),
    };
    match db::load_all(&client).await {
        Ok(docs) => Json(docs.iter().map(cve_view).collect::<Vec<_>>()).into_response(),
        Err(err) => api_error(StatusCode::BAD_GATEWAY, err),
    }
}

async fn cve(Path(cve_id): Path<String>) -> axum::response::Response {
    let client = match db::client() {
        Ok(client) => client,
        Err(err) => return api_error(StatusCode::BAD_GATEWAY, err),
    };
    match db::get_by_id(&client, &cve_id).await {
        Ok(Some(doc)) => Json(cve_view(&doc)).into_response(),
        Ok(None) => api_error(StatusCode::NOT_FOUND, "cve not found"),
        Err(err) => api_error(StatusCode::BAD_GATEWAY, err),
    }
}

async fn knowledge() -> axum::response::Response {
    let client = match db::client() {
        Ok(client) => client,
        Err(err) => return api_error(StatusCode::BAD_GATEWAY, err),
    };
    match db::load_all_knowledge(&client).await {
        Ok(docs) => Json(docs.iter().map(knowledge_view).collect::<Vec<_>>()).into_response(),
        Err(err) => api_error(StatusCode::BAD_GATEWAY, err),
    }
}

async fn knowledge_doc(Path(doc_id): Path<String>) -> axum::response::Response {
    let client = match db::client() {
        Ok(client) => client,
        Err(err) => return api_error(StatusCode::BAD_GATEWAY, err),
    };
    match db::load_all_knowledge(&client).await {
        Ok(docs) => match docs.into_iter().find(|doc| doc.id == doc_id) {
            Some(doc) => Json(knowledge_view(&doc)).into_response(),
            None => api_error(StatusCode::NOT_FOUND, "knowledge document not found"),
        },
        Err(err) => api_error(StatusCode::BAD_GATEWAY, err),
    }
}

async fn search(Query(query): Query<SearchQuery>) -> axum::response::Response {
    let q = query.q.unwrap_or_default();
    let top_k = query.top_k.unwrap_or(25);
    let kind = query.kind.unwrap_or_else(|| "both".to_string());
    let client = match db::client() {
        Ok(client) => client,
        Err(err) => return api_error(StatusCode::BAD_GATEWAY, err),
    };

    let mut results = Vec::new();
    if kind == "cve" || kind == "both" {
        let docs = match db::load_all(&client).await {
            Ok(docs) => docs,
            Err(err) => return api_error(StatusCode::BAD_GATEWAY, err),
        };
        results.extend(
            rank_documents(&docs, &q, top_k)
                .into_iter()
                .map(|hit| SearchRow::Cve {
                    score: hit.score,
                    item: cve_view(hit.document),
                }),
        );
    }
    if kind == "knowledge" || kind == "both" {
        let docs = match db::load_all_knowledge(&client).await {
            Ok(docs) => docs,
            Err(err) => return api_error(StatusCode::BAD_GATEWAY, err),
        };
        results.extend(
            rank_knowledge(&docs, &q, top_k)
                .into_iter()
                .map(|(doc, score)| SearchRow::Knowledge {
                    score,
                    item: knowledge_view(doc),
                }),
        );
    }

    Json(SearchResponse {
        query: q,
        kind,
        count: results.len(),
        results,
    })
    .into_response()
}

fn cve_view(doc: &CveDocument) -> CveView {
    CveView {
        cve_id: doc.cve_id.clone(),
        title: doc.title.clone(),
        description: doc.description.clone(),
        published_at: doc.published_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
        updated_at: doc.updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
        source_url: format!("https://www.cve.org/CVERecord?id={}", doc.cve_id),
    }
}

fn knowledge_view(doc: &zagros::sources::KnowledgeDoc) -> KnowledgeView {
    KnowledgeView {
        id: doc.id.clone(),
        name: doc.name.clone(),
        description: doc.description.clone(),
        source: doc.source.clone(),
        url: safe_source_url(&doc.url).unwrap_or_default(),
        tags: doc.tags.clone(),
    }
}

fn cve_issues(doc: &CveDocument) -> Vec<String> {
    let mut issues = Vec::new();
    if doc.title.trim().is_empty() {
        issues.push("Missing title".into());
    }
    if doc.description.trim().len() < 20 {
        issues.push("Description is missing or too short".into());
    }
    if doc.published_at.is_none() {
        issues.push("Missing published date".into());
    }
    if doc.updated_at.is_none() {
        issues.push("Missing updated date".into());
    }
    issues
}

fn knowledge_issues(doc: &zagros::sources::KnowledgeDoc) -> Vec<String> {
    let mut issues = Vec::new();
    if doc.name.trim().is_empty() {
        issues.push("Missing name".into());
    }
    if doc.description.trim().len() < 20 {
        issues.push("Description is missing or too short".into());
    }
    if doc.source.trim().is_empty() {
        issues.push("Missing source".into());
    }
    if safe_source_url(&doc.url).is_none() {
        issues.push("Missing or unsafe source URL".into());
    }
    issues
}

fn safe_source_url(value: &str) -> Option<String> {
    let url = reqwest::Url::parse(value).ok()?;
    (url.scheme() == "https" && url.host_str().is_some()).then(|| url.to_string())
}

fn api_error(message_status: StatusCode, err: impl std::fmt::Display) -> axum::response::Response {
    (
        message_status,
        Json(serde_json::json!({ "error": err.to_string() })),
    )
        .into_response()
}

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>HelixDB Review Console</title>
<style>
:root{color-scheme:dark;--bg:#071018;--panel:#0d1924;--panel2:#112230;--line:#243746;--text:#e8f0f4;--muted:#90a6b5;--cyan:#31c5c7;--amber:#f5b942;--red:#ff6b6b;--green:#54d18b}*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--text);font:14px/1.45 Inter,ui-sans-serif,system-ui,sans-serif}button,input,select{font:inherit}.topbar{height:64px;display:flex;align-items:center;justify-content:space-between;padding:0 22px;border-bottom:1px solid var(--line);background:#09141dcc;position:sticky;top:0;z-index:5;backdrop-filter:blur(10px)}h1{font-size:17px;margin:0;letter-spacing:.02em}.eyebrow{font-size:11px;text-transform:uppercase;letter-spacing:.13em;color:var(--cyan);font-weight:700}.connection{display:flex;align-items:center;gap:8px;color:var(--muted)}.dot{width:8px;height:8px;border-radius:50%;background:var(--green);box-shadow:0 0 10px var(--green)}.shell{display:grid;grid-template-columns:250px minmax(520px,1fr) 390px;height:calc(100vh - 64px)}aside,main,.inspector{min-width:0;overflow:auto}.sidebar{padding:18px;border-right:1px solid var(--line)}.inspector{padding:18px;border-left:1px solid var(--line);background:#08131c}.label{display:block;color:var(--muted);font-size:11px;text-transform:uppercase;letter-spacing:.1em;margin:18px 0 8px}.nav{display:flex;width:100%;justify-content:space-between;align-items:center;border:0;background:transparent;color:var(--muted);padding:9px 10px;border-radius:8px;cursor:pointer}.nav:hover,.nav.active{background:var(--panel2);color:var(--text)}.count{font-size:11px;color:var(--muted)}.stats{display:grid;grid-template-columns:1fr 1fr;gap:8px}.stat{background:var(--panel);border:1px solid var(--line);border-radius:10px;padding:11px}.stat b{display:block;font-size:20px}.stat span{color:var(--muted);font-size:11px}.toolbar{display:flex;gap:10px;padding:16px 18px;border-bottom:1px solid var(--line);position:sticky;top:0;background:var(--bg);z-index:3}.search{flex:1;position:relative}.search input,select{width:100%;background:var(--panel);color:var(--text);border:1px solid var(--line);border-radius:8px;padding:10px 12px}.search input:focus,select:focus{outline:2px solid #31c5c755;border-color:var(--cyan)}select{width:auto;min-width:130px}.summary{display:flex;justify-content:space-between;padding:14px 18px;color:var(--muted)}.summary strong{color:var(--text)}.table-wrap{padding:0 18px 20px}table{width:100%;border-collapse:separate;border-spacing:0;background:var(--panel);border:1px solid var(--line);border-radius:10px;overflow:hidden}th{text-align:left;color:var(--muted);font-size:11px;text-transform:uppercase;letter-spacing:.08em;padding:11px;border-bottom:1px solid var(--line)}td{padding:12px 11px;border-bottom:1px solid var(--line);vertical-align:top}tr:last-child td{border:0}tbody tr{cursor:pointer}tbody tr:hover,tbody tr.selected{background:var(--panel2)}.id{font:600 12px ui-monospace,SFMono-Regular,monospace;color:var(--cyan);white-space:nowrap}.name{font-weight:600}.snippet{color:var(--muted);font-size:12px;max-width:560px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;margin-top:3px}.badge{display:inline-flex;border:1px solid var(--line);border-radius:999px;padding:2px 7px;font-size:10px;text-transform:uppercase;letter-spacing:.07em;color:var(--muted)}.badge.issue{color:var(--amber);border-color:#74591d}.ok{color:var(--green)}.pager{display:flex;align-items:center;justify-content:flex-end;gap:9px;margin-top:12px}.pager button,.action{background:var(--panel2);color:var(--text);border:1px solid var(--line);border-radius:7px;padding:8px 11px;cursor:pointer}.pager button:disabled{opacity:.4;cursor:default}.empty{padding:70px 20px;text-align:center;color:var(--muted)}.inspect-head{border-bottom:1px solid var(--line);padding-bottom:15px;margin-bottom:15px}.inspect-id{font:700 15px ui-monospace,SFMono-Regular,monospace;color:var(--cyan);margin:5px 0}.inspect-title{font-size:20px;font-weight:700}.section{margin-top:20px}.section h2{font-size:11px;text-transform:uppercase;letter-spacing:.1em;color:var(--muted);margin:0 0 9px}.description{white-space:pre-wrap;word-break:break-word;color:#d7e2e8}.kv{display:grid;grid-template-columns:90px 1fr;gap:8px;padding:7px 0;border-bottom:1px solid #172a37}.kv span:first-child{color:var(--muted)}a{color:var(--cyan)}.issues{border-left:3px solid var(--amber);background:#2a2112;padding:10px 12px;border-radius:4px}.issues div+div{margin-top:5px}details{margin-top:18px}pre{white-space:pre-wrap;word-break:break-word;background:#060d13;border:1px solid var(--line);padding:12px;border-radius:8px;font-size:11px}.loading{opacity:.55}.mobile-inspect{display:none}@media(max-width:1050px){.shell{grid-template-columns:210px 1fr}.inspector{position:fixed;inset:64px 0 0 35%;z-index:6;box-shadow:-20px 0 50px #000;background:#08131c;display:none}.inspector.open{display:block}.mobile-inspect{display:block;float:right}}@media(max-width:720px){.shell{display:block;height:auto}.sidebar{border-right:0;border-bottom:1px solid var(--line)}.toolbar{flex-wrap:wrap}.search{flex-basis:100%}.inspector{inset:64px 0 0 0}.hide-mobile{display:none}}
</style></head><body>
<header class="topbar"><div><div class="eyebrow">Data governance</div><h1>HelixDB Review Console</h1></div><div class="connection"><span class="dot" id="dot"></span><span id="connection">Connecting…</span></div></header>
<div class="shell"><aside class="sidebar"><div class="stats"><div class="stat"><b id="total">—</b><span>Total records</span></div><div class="stat"><b id="issues">—</b><span>Flagged</span></div></div><span class="label">Datasets</span><button class="nav active" data-dataset="all">All records <span class="count" id="allCount"></span></button><button class="nav" data-dataset="cve">CVE records <span class="count" id="cveCount"></span></button><button class="nav" data-dataset="knowledge">Knowledge <span class="count" id="knowledgeCount"></span></button><span class="label">Knowledge sources</span><div id="sources"></div><span class="label">Review note</span><p class="count">Read-only view of values stored in HelixDB. Quality flags are deterministic completeness checks, not security judgments.</p></aside>
<main><div class="toolbar"><div class="search"><input id="query" type="search" placeholder="Find an ID, title, description, tag…" aria-label="Search records"></div><select id="quality" aria-label="Quality filter"><option value="all">All quality</option><option value="issues">Flagged only</option></select><select id="pageSize" aria-label="Rows per page"><option>25</option><option selected>50</option><option>100</option></select></div><div class="summary"><span><strong id="resultCount">0</strong> matching records</span><span id="pageSummary"></span></div><div class="table-wrap"><table><thead><tr><th>Record</th><th>Source</th><th class="hide-mobile">Metadata</th><th>Quality</th></tr></thead><tbody id="rows"></tbody></table><div id="empty" class="empty" hidden>No records match these filters.</div><div class="pager"><button id="prev">Previous</button><span id="pagerText"></span><button id="next">Next</button></div></div></main>
<section class="inspector" id="inspector" aria-label="Record inspector"><button class="action mobile-inspect" id="closeInspector">Close</button><div id="detail" class="empty">Select a record to inspect its complete stored content and source metadata.</div></section></div>
<script>
const state={dataset:'all',source:'all',quality:'all',q:'',page:1,pageSize:50,selected:null};let timer;
const $=id=>document.getElementById(id);const esc=v=>String(v??'').replace(/[&<>'"]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;',"'":'&#39;','"':'&quot;'}[c]));
async function json(url){const r=await fetch(url);if(!r.ok)throw new Error((await r.json().catch(()=>({}))).error||`Request failed (${r.status})`);return r.json()}
async function boot(){try{const s=await json('/api/status');$('connection').textContent='HelixDB connected';$('total').textContent=s.total.toLocaleString();$('issues').textContent=s.issues.toLocaleString();$('allCount').textContent=s.total.toLocaleString();$('cveCount').textContent=s.cves.toLocaleString();$('knowledgeCount').textContent=s.knowledge.toLocaleString();Object.entries(s.sources).forEach(([source,count])=>{const b=document.createElement('button');b.className='nav';b.dataset.source=source;b.innerHTML=`${esc(sourceLabel(source))}<span class="count">${count.toLocaleString()}</span>`;b.onclick=()=>selectSource(source);$('sources').appendChild(b)});await load()}catch(e){$('connection').textContent='HelixDB unavailable';$('dot').style.background='var(--red)';$('rows').innerHTML=`<tr><td colspan="4">${esc(e.message)}</td></tr>`}}
function sourceLabel(v){return v==='attack'?'MITRE ATT&CK':v==='asvs'?'OWASP ASVS':v.toUpperCase()}
function selectDataset(v){state.dataset=v;state.source='all';state.page=1;document.querySelectorAll('.nav').forEach(x=>x.classList.remove('active'));document.querySelector(`[data-dataset="${v}"]`).classList.add('active');load()}
function selectSource(v){state.dataset='knowledge';state.source=v;state.page=1;document.querySelectorAll('.nav').forEach(x=>x.classList.remove('active'));document.querySelector(`[data-source="${v}"]`).classList.add('active');load()}
async function load(){const p=new URLSearchParams({dataset:state.dataset,source:state.source,quality:state.quality,q:state.q,page:state.page,page_size:state.pageSize});$('rows').classList.add('loading');try{const d=await json('/api/review?'+p);state.page=d.page;$('resultCount').textContent=d.total.toLocaleString();$('pageSummary').textContent=`Page ${d.page} of ${d.pages}`;$('pagerText').textContent=`${d.page} / ${d.pages}`;$('prev').disabled=d.page<=1;$('next').disabled=d.page>=d.pages;$('rows').innerHTML='';$('empty').hidden=d.items.length>0;d.items.forEach(item=>$('rows').appendChild(row(item)))}catch(e){$('rows').innerHTML=`<tr><td colspan="4">${esc(e.message)}</td></tr>`}finally{$('rows').classList.remove('loading')}}
function row(item){const tr=document.createElement('tr');tr.tabIndex=0;tr.dataset.key=item.kind+':'+item.id;if(state.selected===tr.dataset.key)tr.className='selected';tr.innerHTML=`<td><div class="id">${esc(item.id)}</div><div class="name">${esc(item.name||'Untitled')}</div><div class="snippet">${esc(item.summary)}</div></td><td><span class="badge">${esc(sourceLabel(item.source))}</span></td><td class="hide-mobile"><div class="snippet">${esc(item.metadata||'—')}</div></td><td>${item.issues.length?`<span class="badge issue">${item.issues.length} flag${item.issues.length>1?'s':''}</span>`:'<span class="ok">Complete</span>'}</td>`;tr.onclick=()=>inspect(item,tr);tr.onkeydown=e=>{if(e.key==='Enter'||e.key===' '){e.preventDefault();inspect(item,tr)}};return tr}
async function inspect(item,tr){state.selected=item.kind+':'+item.id;document.querySelectorAll('tbody tr').forEach(x=>x.classList.toggle('selected',x===tr));$('inspector').classList.add('open');$('detail').className='loading';$('detail').textContent='Loading record…';try{const doc=await json(item.kind==='cve'?`/api/cves/${encodeURIComponent(item.id)}`:`/api/knowledge/${encodeURIComponent(item.id)}`);renderDetail(item,doc)}catch(e){$('detail').className='empty';$('detail').textContent=e.message}}
function renderDetail(item,doc){const isCve=item.kind==='cve',url=isCve?doc.source_url:doc.url,title=isCve?doc.title:doc.name,meta=isCve?[['Published',doc.published_at||'Missing'],['Updated',doc.updated_at||'Missing']]:[['Source',sourceLabel(doc.source)],['Tags',doc.tags||'None stored']];$('detail').className='';$('detail').innerHTML=`<div class="inspect-head"><div class="eyebrow">${isCve?'CVE node':'Knowledge node'}</div><div class="inspect-id">${esc(item.id)}</div><div class="inspect-title">${esc(title||'Untitled')}</div></div>${item.issues.length?`<div class="issues"><strong>Quality flags</strong>${item.issues.map(x=>`<div>• ${esc(x)}</div>`).join('')}</div>`:''}<div class="section"><h2>Stored metadata</h2>${meta.map(([k,v])=>`<div class="kv"><span>${esc(k)}</span><span>${esc(v)}</span></div>`).join('')}<div class="kv"><span>Source URL</span><span>${url?`<a href="${esc(url)}" target="_blank" rel="noopener noreferrer">Open authoritative source ↗</a>`:'Not available'}</span></div></div><div class="section"><h2>Full description</h2><div class="description">${esc(doc.description||'No description stored.')}</div></div><details><summary>Raw API representation</summary><pre>${esc(JSON.stringify(doc,null,2))}</pre></details>`}
document.querySelectorAll('[data-dataset]').forEach(b=>b.onclick=()=>selectDataset(b.dataset.dataset));$('query').oninput=e=>{clearTimeout(timer);timer=setTimeout(()=>{state.q=e.target.value.trim();state.page=1;load()},250)};$('quality').onchange=e=>{state.quality=e.target.value;state.page=1;load()};$('pageSize').onchange=e=>{state.pageSize=Number(e.target.value);state.page=1;load()};$('prev').onclick=()=>{state.page--;load()};$('next').onclick=()=>{state.page++;load()};$('closeInspector').onclick=()=>$('inspector').classList.remove('open');boot();
</script></body></html>"#;
