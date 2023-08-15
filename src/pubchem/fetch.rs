use std::path::Path;

use tokio::io::AsyncWriteExt;

pub async fn down_all_current_sdf(p: impl AsRef<Path>) -> eyre::Result<()> {
    let p = p.as_ref().to_owned();

    if !std::fs::metadata(&p)?.is_dir() {
        return Err(eyre::eyre!("{} must be a directory", p.display()));
    }

    let index_url = "https://ftp.ncbi.nlm.nih.gov/pubchem/Compound/CURRENT-Full/SDF/";
    let index_response = reqwest::get(index_url).await.unwrap();
    assert!(index_response.status().is_success());

    let index_bytes = index_response.bytes().await.unwrap();
    let index = String::from_utf8(index_bytes.to_vec()).unwrap();

    let href_regex = regex::Regex::new("<a href=\"([A-Za-z_0-9.]+)\"").unwrap();

    let captures = href_regex
        .captures_iter(&index)
        .map(|t| t.get(0).unwrap().as_str())
        .map(|m| {
            m.strip_prefix("<a href=\"")
                .unwrap()
                .strip_suffix('"')
                .unwrap()
        })
        .collect::<Vec<_>>();

    let current_dir = std::env::current_dir().unwrap();
    for capture in captures {
        println!("working on {}", capture);
        let mut output_file = tokio::fs::File::create(current_dir.join(&p).join(capture))
            .await
            .unwrap();

        let mut response = reqwest::get(format!("{}/{}", index_url, capture))
            .await
            .unwrap();

        if !response.status().is_success() {
            panic!("whaattt");
        }

        while let Ok(Some(chunk)) = response.chunk().await {
            let res = output_file.write(&chunk).await.unwrap();
            if res == 0 {
                panic!("hmm")
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    // use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn get_index() {}
}
