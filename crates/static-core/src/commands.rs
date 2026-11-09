//! Slash commands typed in the composer: Discord's own text commands such as
//! `/shrug`, and the commands bots and apps offer, which go to Discord as an
//! interaction rather than as a message.

use serde_json::{Value, json};

/// A command a bot or app offers in a channel.
#[derive(Clone, Debug, PartialEq)]
pub struct AppCommand {
    pub id: String,
    pub application_id: String,
    pub version: String,
    pub name: String,
    pub description: String,
    /// The app's name, shown beside the command.
    pub app: String,
    pub options: Vec<CommandOption>,
    /// The command as Discord listed it, sent back with each use.
    pub raw: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommandOption {
    pub name: String,
    pub description: String,
    /// Discord's option type: 1 subcommand, 2 group, 3 text, 4 whole number,
    /// 5 yes or no, 6 user, 7 channel, 8 role, 9 user or role, 10 number,
    /// 11 attachment.
    pub kind: u64,
    pub required: bool,
    /// Its fixed choices: the name shown and the value sent.
    pub choices: Vec<(String, Value)>,
    /// A subcommand's or group's own options.
    pub options: Vec<CommandOption>,
}

const SUBCOMMAND: u64 = 1;
const GROUP: u64 = 2;

fn text(value: &Value, key: &str) -> String {
    match &value[key] {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        _ => String::new(),
    }
}

fn options(value: &Value) -> Vec<CommandOption> {
    value
        .as_array()
        .map(|list| {
            list.iter()
                .map(|option| CommandOption {
                    name: text(option, "name"),
                    description: text(option, "description"),
                    kind: option["type"].as_u64().unwrap_or(3),
                    required: option["required"].as_bool().unwrap_or(false),
                    choices: option["choices"]
                        .as_array()
                        .map(|choices| {
                            choices
                                .iter()
                                .map(|choice| (text(choice, "name"), choice["value"].clone()))
                                .collect()
                        })
                        .unwrap_or_default(),
                    options: options(&option["options"]),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The text commands in an `application-command-index` answer, by name.
pub fn parse_index(index: &Value) -> Vec<AppCommand> {
    let apps: Vec<(String, String)> = index["applications"]
        .as_array()
        .map(|apps| apps.iter().map(|app| (text(app, "id"), text(app, "name"))).collect())
        .unwrap_or_default();
    let mut commands: Vec<AppCommand> = index["application_commands"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter(|command| command["type"].as_u64().unwrap_or(1) == 1)
                .map(|command| {
                    let application_id = text(command, "application_id");
                    AppCommand {
                        id: text(command, "id"),
                        version: text(command, "version"),
                        name: text(command, "name"),
                        description: text(command, "description"),
                        app: apps
                            .iter()
                            .find(|(id, _)| *id == application_id)
                            .map(|(_, name)| name.clone())
                            .unwrap_or_default(),
                        options: options(&command["options"]),
                        raw: command.clone(),
                        application_id,
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    commands.sort_by(|a, b| a.name.cmp(&b.name).then(a.app.cmp(&b.app)));
    commands.dedup_by(|a, b| a.id == b.id);
    commands
}

/// Discord's own text commands: name, what it does.
pub const BUILT_IN: [(&str, &str); 6] = [
    ("shrug", "Appends \u{af}\\_(\u{30c4})_/\u{af} to your message"),
    ("tableflip", "Appends (\u{256f}\u{b0}\u{25a1}\u{b0})\u{256f}\u{fe35} \u{253b}\u{2501}\u{253b} to your message"),
    ("unflip", "Appends \u{252c}\u{2500}\u{252c}\u{30ce}( \u{ba} _ \u{ba}\u{30ce}) to your message"),
    ("me", "Displays text with emphasis"),
    ("spoiler", "Marks your message as a spoiler"),
    ("lenny", "Appends ( \u{361}\u{b0} \u{35c}\u{296} \u{361}\u{b0}) to your message"),
];

/// The message a built-in command makes of `draft`, if it starts with one.
pub fn built_in(draft: &str) -> Option<String> {
    let rest = draft.strip_prefix('/')?;
    let (name, text) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    let text = text.trim();
    let join = |face: &str| if text.is_empty() { face.to_owned() } else { format!("{text} {face}") };
    Some(match name {
        "shrug" => join("\u{af}\\\\_(\u{30c4})\\_/\u{af}"),
        "tableflip" => join("(\u{256f}\u{b0}\u{25a1}\u{b0})\u{256f}\u{fe35} \u{253b}\u{2501}\u{253b}"),
        "unflip" => join("\u{252c}\u{2500}\u{252c}\u{30ce}( \u{ba} _ \u{ba}\u{30ce})"),
        "lenny" => join("( \u{361}\u{b0} \u{35c}\u{296} \u{361}\u{b0})"),
        "me" if !text.is_empty() => format!("_{text}_"),
        "spoiler" if !text.is_empty() => format!("||{text}||"),
        _ => return None,
    })
}

/// The command `draft` names, among `commands`, if it starts with one.
pub fn named<'a>(draft: &str, commands: &'a [AppCommand]) -> Option<&'a AppCommand> {
    let rest = draft.strip_prefix('/')?;
    let name = rest.split(char::is_whitespace).next()?;
    commands.iter().find(|command| command.name == name)
}

/// A short line of a command's options, as `name:` hints, required ones first.
pub fn usage(command: &AppCommand) -> String {
    let mut parts: Vec<String> = Vec::new();
    for option in &command.options {
        match option.kind {
            SUBCOMMAND | GROUP => parts.push(option.name.clone()),
            _ if option.required => parts.push(format!("{}:", option.name)),
            _ => parts.push(format!("[{}:]", option.name)),
        }
    }
    if command.options.iter().any(|option| option.kind == SUBCOMMAND || option.kind == GROUP) {
        parts.join(" | ")
    } else {
        parts.join(" ")
    }
}

/// An option's value from what was typed, by its type.
fn value(option: &CommandOption, typed: &str) -> Result<Value, String> {
    let typed = typed.trim();
    if let Some((_, value)) = option
        .choices
        .iter()
        .find(|(name, value)| name.eq_ignore_ascii_case(typed) || value.as_str().is_some_and(|value| value.eq_ignore_ascii_case(typed)) || value.to_string() == typed)
    {
        return Ok(value.clone());
    }
    if !option.choices.is_empty() {
        let names: Vec<&str> = option.choices.iter().map(|(name, _)| name.as_str()).collect();
        return Err(format!("{} must be one of: {}", option.name, names.join(", ")));
    }
    let id = || {
        typed
            .trim_start_matches("<@&")
            .trim_start_matches("<@!")
            .trim_start_matches("<@")
            .trim_start_matches("<#")
            .trim_end_matches('>')
            .to_owned()
    };
    match option.kind {
        4 => typed
            .parse::<i64>()
            .map(Value::from)
            .map_err(|_| format!("{} needs a whole number", option.name)),
        10 => typed
            .parse::<f64>()
            .map(Value::from)
            .map_err(|_| format!("{} needs a number", option.name)),
        5 => match typed.to_lowercase().as_str() {
            "true" | "yes" | "on" | "1" => Ok(Value::Bool(true)),
            "false" | "no" | "off" | "0" => Ok(Value::Bool(false)),
            _ => Err(format!("{} needs true or false", option.name)),
        },
        6..=9 => {
            let id = id();
            if !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()) {
                Ok(Value::String(id))
            } else {
                Err(format!("{}: pick someone with @ or #", option.name))
            }
        }
        11 => Err(format!("{}: files aren't supported in commands yet", option.name)),
        _ => Ok(Value::String(typed.to_owned())),
    }
}

/// Splits `args` into `name:value` pairs for `options`. Without any `name:`,
/// all of it goes to the first option.
fn pairs<'a>(args: &'a str, options: &'a [CommandOption]) -> Vec<(&'a CommandOption, &'a str)> {
    let mut starts: Vec<(usize, usize, &CommandOption)> = Vec::new();
    for option in options {
        let key = format!("{}:", option.name);
        let mut from = 0;
        while let Some(at) = args[from..].find(&key) {
            let at = from + at;
            if at == 0 || args[..at].ends_with(char::is_whitespace) {
                starts.push((at, at + key.len(), option));
                break;
            }
            from = at + key.len();
        }
    }
    starts.sort_by_key(|(at, ..)| *at);
    if starts.is_empty() {
        return match options.first() {
            Some(first) if !args.trim().is_empty() => vec![(first, args.trim())],
            _ => vec![],
        };
    }
    starts
        .iter()
        .enumerate()
        .map(|(index, (_, value_at, option))| {
            let end = starts.get(index + 1).map_or(args.len(), |(at, ..)| *at);
            (*option, args[*value_at..end].trim())
        })
        .collect()
}

/// The `data.options` of a use of `options`, from what follows the name.
fn filled(args: &str, options: &[CommandOption]) -> Result<Vec<Value>, String> {
    let args = args.trim();
    if options.iter().any(|option| option.kind == SUBCOMMAND || option.kind == GROUP) {
        let (word, rest) = args.split_once(char::is_whitespace).unwrap_or((args, ""));
        let Some(chosen) = options
            .iter()
            .find(|option| (option.kind == SUBCOMMAND || option.kind == GROUP) && option.name == word)
        else {
            let names: Vec<&str> = options.iter().map(|option| option.name.as_str()).collect();
            return Err(format!("Pick one: {}", names.join(", ")));
        };
        return Ok(vec![json!({
            "type": chosen.kind,
            "name": chosen.name,
            "options": filled(rest, &chosen.options)?,
        })]);
    }
    let mut out = Vec::new();
    let given = pairs(args, options);
    for option in options {
        match given.iter().find(|(given, _)| given.name == option.name) {
            Some((_, typed)) if !typed.is_empty() => {
                out.push(json!({"type": option.kind, "name": option.name, "value": value(option, typed)?}));
            }
            _ if option.required => return Err(format!("{} is required", option.name)),
            _ => {}
        }
    }
    Ok(out)
}

/// What Discord's `POST /interactions` takes to run `command` from `draft`,
/// in a channel of a server (`guild`) or a DM. The session id is added by
/// whoever sends it.
pub fn interaction(
    command: &AppCommand,
    draft: &str,
    guild: Option<&str>,
    channel: &str,
    nonce: &str,
) -> Result<Value, String> {
    let args = draft
        .strip_prefix('/')
        .and_then(|rest| rest.strip_prefix(command.name.as_str()))
        .unwrap_or("");
    let options = filled(args, &command.options)?;
    let mut body = json!({
        "type": 2,
        "application_id": command.application_id,
        "channel_id": channel,
        "nonce": nonce,
        "analytics_location": "slash_ui",
        "data": {
            "version": command.version,
            "id": command.id,
            "name": command.name,
            "type": 1,
            "options": options,
            "application_command": command.raw,
            "attachments": [],
        },
    });
    if let Some(guild) = guild {
        body["guild_id"] = json!(guild);
    }
    Ok(body)
}

/// A Discord snowflake for now, as the nonce of an interaction.
pub fn nonce(now_ms: u64) -> String {
    (now_ms.saturating_sub(1_420_070_400_000) << 22).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index() -> Value {
        json!({
            "applications": [{"id": "9", "name": "Music"}],
            "application_commands": [
                {"id": "1", "application_id": "9", "version": "5", "type": 1, "name": "play",
                 "description": "Play a song",
                 "options": [{"type": 3, "name": "query", "description": "Song", "required": true},
                             {"type": 5, "name": "next", "description": "Queue next"}]},
                {"id": "2", "application_id": "9", "version": "5", "type": 1, "name": "leave", "description": "Leave"},
                {"id": "3", "application_id": "9", "version": "5", "type": 2, "name": "User menu"},
                {"id": "4", "application_id": "9", "version": "5", "type": 1, "name": "loop", "description": "Loop",
                 "options": [{"type": 1, "name": "song", "description": "One song", "options": []},
                             {"type": 1, "name": "queue", "description": "All",
                              "options": [{"type": 4, "name": "times", "description": "n", "required": true}]}]},
            ]
        })
    }

    #[test]
    fn reads_text_commands_only() {
        let commands = parse_index(&index());
        let names: Vec<&str> = commands.iter().map(|command| command.name.as_str()).collect();
        assert_eq!(names, ["leave", "loop", "play"]);
        assert_eq!(commands[2].app, "Music");
    }

    #[test]
    fn fills_options() {
        let commands = parse_index(&index());
        let play = named("/play never gonna give you up", &commands).unwrap();
        let body = interaction(play, "/play never gonna give you up", Some("7"), "8", "1").unwrap();
        assert_eq!(body["data"]["options"], json!([{"type": 3, "name": "query", "value": "never gonna give you up"}]));
        assert_eq!(body["guild_id"], "7");
        let body = interaction(play, "/play query:abc next:yes", None, "8", "1").unwrap();
        assert_eq!(
            body["data"]["options"],
            json!([{"type": 3, "name": "query", "value": "abc"}, {"type": 5, "name": "next", "value": true}])
        );
        assert!(body.get("guild_id").is_none());
        assert_eq!(interaction(play, "/play", None, "8", "1").unwrap_err(), "query is required");
        let leave = named("/leave", &commands).unwrap();
        assert_eq!(interaction(leave, "/leave", None, "8", "1").unwrap()["data"]["options"], json!([]));
    }

    #[test]
    fn fills_subcommands() {
        let commands = parse_index(&index());
        let looped = named("/loop queue times:3", &commands).unwrap();
        let body = interaction(looped, "/loop queue times:3", None, "8", "1").unwrap();
        assert_eq!(
            body["data"]["options"],
            json!([{"type": 1, "name": "queue", "options": [{"type": 4, "name": "times", "value": 3}]}])
        );
        assert!(interaction(looped, "/loop", None, "8", "1").is_err());
    }

    #[test]
    fn built_ins() {
        assert_eq!(built_in("/me waves").unwrap(), "_waves_");
        assert_eq!(built_in("/spoiler secret").unwrap(), "||secret||");
        assert!(built_in("/shrug ok").unwrap().starts_with("ok "));
        assert!(built_in("/play x").is_none());
        assert!(built_in("hello").is_none());
    }
}
