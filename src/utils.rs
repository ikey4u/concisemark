use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub fn escape_to_html(text: &str) -> String {
    let mut html = String::new();
    for ch in text.chars() {
        match ch {
            '&' => {
                html.push_str("&amp;");
            }
            '>' => {
                html.push_str("&gt;");
            }
            '<' => {
                html.push_str("&lt;");
            }
            _ => {
                html.push(ch);
            }
        }
    }
    html
}

pub fn escape_to_tex(text: &str) -> String {
    let mut content = String::new();
    for ch in text.chars() {
        match ch {
            '$' => {
                content.push_str(r#"\$"#);
            }
            '%' => {
                content.push_str(r#"\%"#);
            }
            '`' => content.push_str(r#"\verb|`|"#),
            '\\' => content.push_str(r#"\textbackslash"#),
            _ => {
                content.push(ch);
            }
        }
    }
    content
}

/// Download image from `url` and save it into directory `dir` with name `name`,
/// the image suffix is guessed from its content type.
pub fn download_image_fs<S1, S2, P>(
    url: S1,
    dir: P,
    name: S2,
) -> Option<PathBuf>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
    P: AsRef<Path>,
{
    let url = url.as_ref();
    let dir = dir.as_ref();
    let name = name.as_ref();
    if let Some((content_type, data)) = download_image(url) {
        let suffix = match content_type.as_str() {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "image/svg+xml" => "svg",
            _ => ".unknwon",
        };
        // TODO: add more name safety checking
        let name = name
            .replace("%", "_")
            .replace("/", "_")
            .replace("\\", "_")
            .replace(".", "_");
        let output_path = dir.join(format!("{name}.{suffix}"));
        let mut f = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(&output_path)
            .ok()?;
        f.write(&data[..]).ok()?;
        Some(output_path)
    } else {
        return None;
    }
}

/// Download image from url and return its content type and data if success
pub fn download_image<S: AsRef<str>>(url: S) -> Option<(String, Vec<u8>)> {
    let url = url.as_ref();
    let mut data: Vec<u8> = vec![];
    match ureq::get(url.as_ref()).call() {
        Ok(resp) => {
            let content_type = resp.content_type().to_owned();
            // max size is limited to 10MB
            if let Err(e) =
                resp.into_reader().take(10_000_000).read_to_end(&mut data)
            {
                // TODO: better error handling
                log::error!("failed to read media data into buffer: {e:?}");
            }
            return Some((content_type, data));
        }
        Err(e) => {
            println!("error: {e:?} ==> {url}");
            // TODO: better error handling
            log::error!("failed to download media {} with error {e:?}", url);
            return None;
        }
    }
}
