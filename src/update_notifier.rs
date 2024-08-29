use curl::easy::{Easy, List};
use serde_derive::Deserialize;

pub const REGISTRY_URL: &str = "https://crates.io";

#[derive(Debug, thiserror::Error)]
enum ErrorKind {
    #[error("Error while parsing json: {0}")]
    UnableToParseJson(String),
    #[error("Error received from registry: {0}")]
    RegistryError(String),
}

#[derive(Deserialize, Debug, Clone)]
struct VersionResponse {
    versions: Option<Vec<Version>>,
    errors: Option<Vec<JsonError>>,
}

#[derive(Deserialize, Debug, Clone)]
struct JsonError {
    detail: String,
}

#[derive(Deserialize, Debug, Clone)]
struct Version {
    num: String,
}

fn get_latest_from_json(resp: &VersionResponse) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(versions) = &resp.versions {
        match versions.first() {
            Some(version) => Ok(version.num.clone()),
            None => Err(ErrorKind::UnableToParseJson("Versions array is empty".to_string()).into()),
        }
    } else if let Some(errors) = &resp.errors {
        match errors.first() {
            Some(error) => Err(ErrorKind::RegistryError(error.detail.clone()).into()),
            None => Err(
                ErrorKind::UnableToParseJson("No errors in the errors array".to_string()).into(),
            ),
        }
    } else {
        Err(ErrorKind::UnableToParseJson(
            "Invalid json response, does not have versions or errors".to_string(),
        )
        .into())
    }
}

fn get_latest_version(
    crate_name: &str,
    registry_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // We use curl-rust here to save us importing a bunch of dependencies pulled in with reqwest
    // We're okay with a blocking api since it's only one small request
    let mut easy = Easy::new();

    let url = format!("{}/api/v1/crates/{}/versions", registry_url, crate_name);
    easy.url(&url)?;
    let mut list = List::new();
    list.append("User-Agent: Update-notifier (teshaq@mozilla.com)")?;
    easy.http_headers(list)?;
    let mut resp_buf = Vec::new();
    // Create a different lifetime for `transfer` since it
    // borrows resp_buf in it's closure

    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            resp_buf.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }
    let resp = std::str::from_utf8(&resp_buf)?;
    let json_resp = serde_json::from_str(resp)?;
    get_latest_from_json(&json_resp)
}

fn generate_notice(name: &str, current_version: &str, latest_version: &str) -> String {
    let line_1 = format!(
        "A new version of {} is available! {} → {}",
        name, current_version, latest_version
    );

    let suggestion = format!("cargo install {}", name);
    let line_2 = format!("Use `{}` to install version {}", suggestion, latest_version);

    let url = format!("{}/crates/{}", REGISTRY_URL, name);
    let line_3 = format!("Check {} for more details", url);
    let mut border_line = String::from("\n───────────────────────────────────────────────────────");
    let extension = "─";
    for _ in 0..name.len() {
        border_line.push_str(extension);
    }
    border_line.push('\n');
    format!(
        "{}
    {}
    {}
    {}
    {}",
        border_line, line_1, line_2, line_3, border_line
    )
}

fn print_notice(name: &str, current_version: &str, latest_version: &str) {
    print!("{}", generate_notice(name, current_version, latest_version));
}

pub fn check_latest_version(
    name: &str,
    current_version: &str,
    registry_url: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let latest_version = get_latest_version(name, registry_url)?;
    if latest_version != current_version {
        print_notice(name, current_version, &latest_version);
        return Ok(false);
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_latest_version() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/api/v1/crates/asdev/versions")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"
            {"versions" : [
                {
                    "id": 229435,
                    "crate": "asdev",
                    "num": "0.1.3"
                }
            ]}"#,
            )
            .create();
        let latest_version = get_latest_version("asdev", &server.url()).unwrap();
        _m.expect(1).assert();
        assert_eq!(latest_version, "0.1.3")
    }

    #[test]
    fn test_no_crates_io_entry() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock(
                "GET",
                "/api/v1/crates/kefjhkajvcnklsajdfhwksajnceknc/versions",
            )
            .with_status(404)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"
                {"errors":[{"detail":"Not Found"}]}"#,
            )
            .create();
        let latest_version = get_latest_version("kefjhkajvcnklsajdfhwksajnceknc", &server.url())
            .expect_err("Should be an error");
        _m.expect(1).assert();
        assert_eq!(
            latest_version.to_string(),
            ErrorKind::RegistryError("Not Found".to_string()).to_string()
        );
    }

    #[test]
    fn test_same_version() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/api/v1/crates/sameVersion/versions")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"
            {"versions" : [
                {
                    "id": 229435,
                    "crate": "sameVersion",
                    "num": "0.1.3"
                }
            ]}"#,
            )
            .create();
        check_latest_version("sameVersion", "0.1.3", &server.url()).unwrap();
        _m.expect(1).assert();
    }

    #[test]
    fn test_not_update_available() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/api/v1/crates/noUpdate/versions")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"
            {"versions" : [
                {
                    "id": 229435,
                    "crate": "noUpdate",
                    "num": "0.1.3"
                }
            ]}"#,
            )
            .create();
        check_latest_version("noUpdate", "0.1.2", &server.url()).unwrap();
        _m.expect(1).assert();
    }

    #[test]
    fn test_output() {
        assert_eq!(generate_notice("asdev", "0.1.2", "0.1.3"), "\n────────────────────────────────────────────────────────────\n\n    A new version of asdev is available! 0.1.2 → 0.1.3\n    Use `cargo install asdev` to install version 0.1.3\n    Check https://crates.io/crates/asdev for more details\n    \n────────────────────────────────────────────────────────────\n");
    }

    #[test]
    fn test_interval_not_exceeded() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/api/v1/crates/notExceeded/versions")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"
            {"versions" : [
                {
                    "id": 229435,
                    "crate": "notExceeded",
                    "num": "0.1.3"
                }
            ]}"#,
            )
            .create();
        check_latest_version("notExceeded", "0.1.2", &server.url()).unwrap();
        _m.expect(1).assert()
    }

    #[test]
    fn test_interval_exceeded() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/api/v1/crates/intervalExceeded/versions")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"
            {"versions" : [
                {
                    "id": 229435,
                    "crate": "intervalExceeded",
                    "num": "0.1.3"
                }
            ]}"#,
            )
            .create();

        check_latest_version("intervalExceeded", "0.1.2", &server.url()).unwrap();
        _m.expect(1).assert()
    }
}
