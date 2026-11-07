//! This account's connections, and the browser's answer to a login Static
//! started. Profile connections omit the ids and switches needed here.

use anyhow::{Result, bail};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Connection {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub visibility: u8,
    #[serde(default)]
    pub show_activity: bool,
    #[serde(default)]
    pub revoked: bool,
}

/// The settings the Connections screen offers, one field per request.
#[derive(Clone, Copy)]
pub enum Setting {
    Profile(bool),
    Activity(bool),
}

impl Setting {
    pub(crate) fn body(self) -> Value {
        match self {
            Self::Profile(shown) => json!({"visibility": shown}),
            Self::Activity(shown) => json!({"show_activity": shown}),
        }
    }
}

/// Kept in memory until the browser finishes, never logged or saved.
pub struct Authorization {
    pub url: String,
    pub state: String,
}

impl Authorization {
    pub(crate) fn parse(address: String) -> Result<Self> {
        let url = Url::parse(&address).map_err(|_| anyhow::anyhow!("Invalid login URL."))?;
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            bail!("Discord returned an invalid login URL.");
        }
        // Steam puts Discord's state in OpenID's return address.
        let state = state_in(&url, 0)
            .filter(|state| !state.is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Discord's login URL has no state. This provider needs a different login flow."
                )
            })?;
        Ok(Self {
            url: address,
            state,
        })
    }
}

fn state_in(url: &Url, depth: u8) -> Option<String> {
    if depth > 2 {
        return None;
    }
    let pairs: Vec<_> = url.query_pairs().collect();
    let states: Vec<_> = pairs.iter().filter(|(key, _)| key == "state").collect();
    if states.len() == 1 {
        return Some(states[0].1.to_string());
    }
    if !states.is_empty() {
        return None;
    }
    pairs
        .iter()
        .filter(|(key, _)| matches!(key.as_ref(), "openid.return_to" | "redirect_uri"))
        .filter_map(|(_, value)| Url::parse(value).ok())
        .find_map(|url| state_in(&url, depth + 1))
}

/// Only an answer from Discord for this provider and this pending state can
/// finish the connection. Errors never include the pasted URL or its code.
pub fn callback(kind: &str, expected_state: &str, address: &str) -> Result<Value> {
    if address.len() > 32_768 {
        bail!("The redirect URL is too long.");
    }
    let url = Url::parse(address.trim())
        .map_err(|_| anyhow::anyhow!("Paste the complete Discord redirect URL."))?;
    let path = format!("/connections/{kind}");
    if url.scheme() != "https"
        || url.host_str() != Some("discord.com")
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || (url.path() != path && url.path() != format!("{path}/error"))
    {
        bail!("Use the discord.com redirect URL for the provider you selected.");
    }
    let mut params = Map::new();
    for (key, value) in url.query_pairs() {
        if params
            .insert(key.into_owned(), Value::String(value.into_owned()))
            .is_some()
        {
            bail!("The redirect URL contains duplicate parameters.");
        }
    }
    if params.contains_key("error") {
        bail!("The provider declined the login. Start a new connection and try again.");
    }
    if expected_state.is_empty()
        || params.get("state").and_then(Value::as_str) != Some(expected_state)
    {
        bail!("This redirect does not match the login Static started. Start a new connection.");
    }
    let code = params
        .get("oauth_verifier")
        .or_else(|| params.get("code"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let openid: Map<String, Value> = params
        .iter()
        .filter(|(key, _)| key.starts_with("openid."))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    if code.is_empty() && (kind != "steam" || openid.is_empty()) {
        bail!(
            "The redirect URL has no authorization code. If the browser already finished, use Refresh."
        );
    }
    let mut body = json!({
        "code": code,
        "state": expected_state,
        "insecure": false,
        "friend_sync": matches!(kind, "facebook" | "contacts"),
    });
    if !openid.is_empty() {
        body["openid_params"] = Value::Object(openid);
    }
    if let Some(iss) = params.get("iss") {
        body["iss"] = iss.clone();
    }
    Ok(body)
}

/// Escapes an account id as one path segment, including Steam's and domains'.
pub(crate) fn path(kind: &str, id: &str) -> Result<String> {
    if kind.is_empty()
        || !kind
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' || c == b'-')
        || id.is_empty()
        || matches!(id, "." | "..")
    {
        bail!("Invalid connection.");
    }
    let id: String = url::form_urlencoded::byte_serialize(id.as_bytes()).collect();
    Ok(format!("/users/@me/connections/{kind}/{id}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_accounts_settings_without_their_tokens() {
        let connection: Connection = serde_json::from_value(json!({
            "id": "123", "type": "spotify", "name": "Alice", "visibility": 0,
            "show_activity": true, "access_token": "not-kept"
        }))
        .unwrap();
        assert!(!connection.revoked);
        assert_eq!(connection.visibility, 0);
        assert!(connection.show_activity);
        assert_eq!(Setting::Profile(true).body(), json!({"visibility": true}));
        assert_eq!(
            Setting::Activity(false).body(),
            json!({"show_activity": false})
        );
        assert_eq!(
            path("domain", "alice.example/a?b").unwrap(),
            "/users/@me/connections/domain/alice.example%2Fa%3Fb"
        );
    }

    #[test]
    fn accepts_only_the_login_this_client_started() {
        let url = "https://discord.com/connections/github?code=abc%2B123&state=ours&iss=https%3A%2F%2Fgithub.com";
        let body = callback("github", "ours", url).unwrap();
        assert_eq!(body["code"], "abc+123");
        assert_eq!(body["iss"], "https://github.com");
        assert_eq!(body["insecure"], false);
        for bad in [
            url.replace("discord.com/", "discord.com.evil/"),
            url.replace("https:", "http:"),
            url.replace("github?", "spotify?"),
            format!("{url}&state=ours"),
            url.replace("state=ours", "state=other"),
            format!("{url}#extra"),
            url.replace("code=abc%2B123", "error=access_denied"),
        ] {
            assert!(callback("github", "ours", &bad).is_err());
        }
    }

    #[test]
    fn steam_uses_the_return_address_and_openid_parameters() {
        let authorization = Authorization::parse("https://steamcommunity.com/openid/login?openid.return_to=https%3A%2F%2Fdiscord.com%2Fconnections%2Fsteam%3Fstate%3Dours".into()).unwrap();
        assert_eq!(authorization.state, "ours");
        let body = callback(
            "steam",
            "ours",
            "https://discord.com/connections/steam?state=ours&openid.mode=id_res&openid.sig=signed",
        )
        .unwrap();
        assert_eq!(body["code"], "");
        assert_eq!(body["openid_params"]["openid.sig"], "signed");
        assert!(Authorization::parse("https://example.com/no-state".into()).is_err());
        assert!(
            callback(
                "steam",
                "ours",
                "https://discord.com/connections/steam?state=ours"
            )
            .is_err()
        );
    }
}
