use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub headers: Vec<Header>,
    pub body: Option<String>,
}

pub fn json_to_toml(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let req: Request = serde_json::from_str(input)?;
    let toml = toml::to_string_pretty(&req)?;
    Ok(toml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_json_to_toml() {
        let json = r#"
        {
            "method": "GET",
            "url": "https://example.com",
            "headers": [
                { "name": "Accept", "value": "application/json" }
            ],
            "body": null
        }
        "#;

        let toml = json_to_toml(json).unwrap();
        assert!(toml.contains("method = \"GET\""));
        assert!(toml.contains("url = \"https://example.com\""));
    }
}