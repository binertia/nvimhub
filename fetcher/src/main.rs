use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::Error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

// -----------------------------------------

#[derive(Deserialize, Serialize, Clone)]
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

// FIX: tokio panic when api request is max cap
#[tokio::main]
async fn main() {
    let data = warp::path("data").and_then(handle_fetch);

    warp::serve(data).run(([127, 0, 0, 1], 8081)).await;
}

async fn fetch_github_repositories() -> Result<Vec<Repository>, Error> {
    let base_url = "https://api.github.com/search/repositories?q=nvim+plugin";
    let client = reqwest::Client::new();
    let all_repos = Arc::new(Mutex::new(Vec::<Repository>::new()));

    // fetch 120 repo
    let fetches = (1..=4)
        .map(|page| {
            let url = format!("{}&per_page=100&page={}", base_url, page);
            let client = &client;
            async move {
                println!("Fetching page: {}", page); // log
                let response = client
                    .get(url)
                    .header("User-Agent", "Rust reqwest") // github neeeeeed
                    .send()
                    .await?;
                assert!(
                    response.status().is_success(),
                    "async move assert: github fetch request failed"
                );
                let search_result: SearchResult = response.json().await?;
                assert!(
                    !search_result.items.is_empty(),
                    "async move assert: no repositories found"
                );
                let repos: Vec<Repository> = search_result
                    .items
                    .into_iter()
                    .map(|repo| Repository {
                        name: repo.name,
                        description: repo.description,
                        html_url: repo.html_url,
                        stargazers_count: repo.stargazers_count,
                    })
                    .collect();

                Ok::<Vec<Repository>, Error>(repos)
            }
        })
        .collect::<FuturesUnordered<_>>();

    fetches
        .for_each_concurrent(None, |res| async {
            if let Ok(repos) = res {
                let mut repos_lock = all_repos.lock().await;
                repos_lock.extend(repos);
            } else if let Err(e) = res {
                eprintln!("fetches.for_each_con :error fetching page: {}", e);
            }
        })
        .await;

    let repos_lock = all_repos.lock().await;
    let mut all_repos = repos_lock.clone(); // clone the vector from the lock

    // sort by star count
    all_repos.sort_by(|a, b| a.stargazers_count.cmp(&b.stargazers_count));
    Ok(all_repos)
}

async fn handle_fetch() -> Result<impl warp::Reply, warp::Rejection> {
    match fetch_github_repositories().await {
        Ok(repo) => Ok(warp::reply::json(&repo)),
        Err(_) => Err(warp::reject::not_found()),
    }
}
