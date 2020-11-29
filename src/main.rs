use lopdf::Document;
use regex::Regex;
use std::fs;
use std::io::copy;
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
    const BASE: &str = "https://papacambridge.com/a-and-as-level-subjects/";

    let subjects: Vec<String> = {
        let response = reqwest::get(BASE).await?;

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
            let response = reqwest::get(&paper[..]).await?;

            println!("{}", paper);

            if response.status() != 200 {
                println!(
                    "Failed GET request, status: {}, origin: {}",
                    response.status(),
                    paper
                );
            }

            let re = Regex::new("folder\\.png.*?class=.headingwrap.").unwrap();
            let body = response.text().await?;

            // Check if page includes folders, if so visits subsites
            if let Some(_) = re.captures(body.as_ref()) {
                let re = Regex::new("(.{9}?)\" class=.clearfix.").unwrap();

                for subsite in re.captures_iter(body.as_ref()) {
                    let subsite = [&paper[..], &subsite[1]].concat();
                    let response = reqwest::get(&subsite[..]).await?;

                    println!("{}", subsite);

                    if response.status() != 200 {
                        println!(
                            "Failed GET request, status: {}, origin: {}",
                            response.status(),
                            subsite
                        );
                    }

                    let re = Regex::new("href=\"\\.\\./\\.\\./(.*?((upload/).*?))\">Download</a>")
                        .unwrap();
                    println!("Reached regex for the download!");

                    for file in re.captures_iter(response.text().await?.as_ref()) {
                        let extension = [&file[1], &file[3], &file[2]].concat();
                        let file_path = [BASE, &extension[..]].concat();
                        let response = reqwest::get(&file_path[..]).await?;

                        if response.status() != 200 {
                            println!(
                                "Failed GET request, status: {}, origin: {}",
                                response.status(),
                                extension
                            );
                        }

                        let local_file_name = ["data/", &file[2]].concat();
                        let content = response.text().await?;
                        let mut destination = fs::File::create(&local_file_name[..])
                            .expect("Failed to create file in data dir...");

                        copy(&mut content.as_bytes(), &mut destination)
                            .expect("Failed to write to file in data dir...");
                    }
                }
            } else {
                // TODO: Must replicate the functionality added within subsites, currently it will
                // only download past papers if they're categorized into two or more exams per
                // year, if cambridge released only one set of exams that year for a subject it
                // will ignore the paper.
                unimplemented!();
            };
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir("data")
        .map_err(|e| println!("Failed to create \"data\" folder: {}", e))
        .unwrap();
    let _papers = read_papers("data");

    //for paper in _papers {
    //    for page in paper {
    //        println!("{}", page);
    //    }
    //}

    pull_papers_cambridge().await?;

    Ok(())
}
