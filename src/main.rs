use lopdf::Document;
use regex::Regex;
use std::fs;
use std::io::prelude::*;

pub fn read_papers(dir: &str) -> Vec<Vec<String>> {
    let mut papers: Vec<Vec<String>> = Default::default(); // outer: papers, inner: pages

    let documents: Vec<String> = fs::read_dir(dir)
        .map(|f| {
            f.map(|e| {
                e.unwrap()
                    .path()
                    .into_os_string()
                    .into_string()
                    .unwrap_or("".to_string())
            })
        })
        .unwrap()
        .collect();

    for document in documents {
        println!("{}", document);
        let paper: Document =
            Document::load("data/specimen-paper.pdf").expect("Failed to load paper");
        let page_count: usize = paper.get_pages().len();
        let mut pages: Vec<String> = Default::default();

        pages.reserve(page_count);

        for (_, page_id) in paper.get_pages() {
            let page = paper
                .get_page_content(page_id)
                .unwrap_or("\nfailed...\n".as_bytes().to_vec());
            pages.push(Document::decode_text(None, &page));
            //pages.push(page.iter().map(|&c| c as char).collect::<String>());
        }
        papers.push(pages);
    }
    papers
}

async fn pull_papers_cambridge() -> Result<(), reqwest::Error> {
    let subjects: Vec<String> = {
        let response = reqwest::get("https://papacambridge.com/a-and-as-level-subjects/").await?;

        if response.status() != 200 {
            println!("Failed GET request, status: {}", response.status());
        }

        let re = Regex::new("href=\"(.*?)\".*?class=\"sublistsyllcode\">").unwrap();

        // Copies all cases of the first capture group (https://...) into a Vec
        re.captures_iter(response.text().await?.as_ref())
            .map(|res| res[1].to_string())
            .collect()
    };

    let mut log = fs::File::create("data/papers.log").unwrap();

    for subject in subjects {
        let papers: Vec<String> = {
            let response = reqwest::get(&subject[..]).await?;

            if response.status() != 200 {
                println!(
                    "Failed GET request, status: {}, origin: {}",
                    response.status(),
                    subject
                );
            }

            let re = Regex::new("class=\"clearfix\".*?href=\"(.*?)\"").unwrap();
            re.captures_iter(response.text().await?.as_ref())
                .map(|res| res[1].to_string())
                .collect()
        };
        log.write(papers.join("\n").as_bytes()).unwrap();
        for paper in papers {
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match fs::create_dir("data") {
        Err(_) => (),
        _ => (),
    };
    let _papers = read_papers("data");

    //for paper in _papers {
    //    for page in paper {
    //        println!("{}", page);
    //    }
    //}

    pull_papers_cambridge().await?;

    Ok(())
}
