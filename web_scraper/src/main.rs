use reqwest::Error;
use serde::{Deserialize, Serialize};
use warp::Filter;

// -----------------------------------------

#[derive(Deserialize, Serialize)]
struct Repository {
    name: String,
    description: Option<String>,
    html_url: String,
    #[serde(skip_serializing)] // skip star
    stargazers_count: u32,
}

#[derive(Deserialize)]
struct SearchResult {
    items: Vec<RepositoryWithStars>,
}

#[derive(Deserialize)]
struct RepositoryWithStars {
    name: String,
    description: Option<String>,
    html_url: String,
    stargazers_count: u32,
}

// ----------------------------------------

#[tokio::main]
async fn main() {
    let data = warp::path("data").and_then(handle_fetch);

    warp::serve(data).run(([127, 0, 0, 1], 8081)).await;
}

async fn fetch_github_repositories() -> Result<Vec<Repository>, Error> {
    let url = "https://api.github.com/search/repositories?q=nvim+plugin";

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "Rust reqwest") // github neeeeeed
        .send()
        .await?;

    assert!(
        response.status().is_success(),
        "github fetch request failed"
    );

    let search_result: SearchResult = response.json().await?;

    assert!(!search_result.items.is_empty(), "No repositories found");

    // to map
    let mut repos: Vec<Repository> = search_result
        .items
        .into_iter()
        .map(|repo| Repository {
            name: repo.name,
            description: repo.description,
            html_url: repo.html_url,
            stargazers_count: repo.stargazers_count,
        })
        .collect();

    // sort by star count
    repos.sort_by(|a, b| a.stargazers_count.cmp(&b.stargazers_count));

    Ok(repos)
}

async fn handle_fetch() -> Result<impl warp::Reply, warp::Rejection> {
    match fetch_github_repositories().await {
        Ok(repo) => Ok(warp::reply::json(&repo)),
        Err(_) => Err(warp::reject::not_found()),
    }
}
