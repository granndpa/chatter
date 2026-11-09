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
    /// The app's icon address, if it has one.
    pub icon: Option<String>,
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
    let apps: Vec<(String, String, String)> = index["applications"]
        .as_array()
        .map(|apps| apps.iter().map(|app| (text(app, "id"), text(app, "name"), text(app, "icon"))).collect())
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
                            .find(|(id, ..)| *id == application_id)
                            .map(|(_, name, _)| name.clone())
                            .unwrap_or_default(),
                        icon: apps
                            .iter()
                            .find(|(id, _, icon)| *id == application_id && !icon.is_empty())
                            .map(|(id, _, icon)| format!("https://cdn.discordapp.com/app-icons/{id}/{icon}.png?size=64")),
                        options: options(&command["options"]),
                        raw: command.clone(),
                        application_id,
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    commands.sort_by(|a, b| a.app.to_lowercase().cmp(&b.app.to_lowercase()).then(a.name.cmp(&b.name)));
    commands.dedup_by(|a, b| a.id == b.id);
    commands
}

/// The application id the built-in commands go by.
pub const BUILT_IN: &str = "builtin";

fn option(name: &str, description: &str, kind: u64, required: bool) -> CommandOption {
    CommandOption {
        name: name.into(),
        description: description.into(),
        kind,
        required,
        choices: Vec::new(),
        options: Vec::new(),
    }
}

/// Discord's own commands that Chatter carries out itself: in a server's
/// channel (`in_server`) or a DM.
pub fn built_ins(in_server: bool) -> Vec<AppCommand> {
    let message = || option("message", "Your message", 3, false);
    let mut list = vec![
        ("lenny", "Appends ( \u{361}\u{b0} \u{35c}\u{296} \u{361}\u{b0}) to your message.", vec![message()]),
        ("me", "Displays text with emphasis.", vec![option("message", "Your message", 3, true)]),
        ("shrug", "Appends \u{af}\\_(\u{30c4})_/\u{af} to your message.", vec![message()]),
        ("spoiler", "Marks your message as a spoiler.", vec![option("message", "Your message", 3, true)]),
        ("tableflip", "Appends (\u{256f}\u{b0}\u{25a1}\u{b0})\u{256f}\u{fe35} \u{253b}\u{2501}\u{253b} to your message.", vec![message()]),
        ("unflip", "Appends \u{252c}\u{2500}\u{252c}\u{30ce}( \u{ba} _ \u{ba}\u{30ce}) to your message.", vec![message()]),
    ];
    if in_server {
        list.extend([
            ("ban", "Ban a user from this server.", vec![
                option("user", "The user to ban", 6, true),
                option("delete_messages", "Delete their messages from the last hours (0 to 168)", 4, false),
                option("reason", "The reason for the ban", 3, false),
            ]),
            ("kick", "Kick a user from this server.", vec![
                option("user", "The user to kick", 6, true),
                option("reason", "The reason for the kick", 3, false),
            ]),
            ("nick", "Change your nickname on this server.", vec![option("new_nick", "Your new nickname, empty resets it", 3, false)]),
            ("timeout", "Time out a user.", vec![
                option("user", "The user to time out", 6, true),
                option("duration", "How long: 60s, 5m, 10m, 1h, 1d or 1w", 3, true),
                option("reason", "The reason for the timeout", 3, false),
            ]),
        ]);
    } else {
        let mut silent = option("silent", "Leave without notifying other members", 5, false);
        silent.choices = vec![("True".into(), Value::Bool(true)), ("False".into(), Value::Bool(false))];
        list.push(("leave", "Leave Group", vec![silent]));
    }
    list.sort_by_key(|(name, ..)| *name);
    list.into_iter()
        .map(|(name, description, options)| AppCommand {
            id: format!("{BUILT_IN}-{name}"),
            application_id: BUILT_IN.into(),
            version: String::new(),
            name: name.into(),
            description: description.into(),
            app: "Built-In".into(),
            icon: None,
            options,
            raw: Value::Null,
        })
        .collect()
}

/// What a built-in command does.
#[derive(Clone, Debug, PartialEq)]
pub enum BuiltIn {
    /// Send this message.
    Text(String),
    Nick(String),
    Leave,
    Kick { user: String, reason: String },
    Ban { user: String, delete_seconds: u32, reason: String },
    Timeout { user: String, seconds: u64, reason: String },
}

/// Seconds from a duration such as `60s`, `5m`, `1h`, `1d` or `1w`; a bare
/// number is minutes.
fn seconds(typed: &str) -> Option<u64> {
    let typed = typed.trim().to_lowercase();
    let (number, unit) = typed.split_at(typed.find(|c: char| !c.is_ascii_digit()).unwrap_or(typed.len()));
    let number: u64 = number.parse().ok()?;
    let unit = match unit.trim() {
        "s" | "sec" | "secs" | "seconds" => 1,
        "" | "m" | "min" | "mins" | "minutes" => 60,
        "h" | "hour" | "hours" => 3600,
        "d" | "day" | "days" => 86_400,
        "w" | "week" | "weeks" => 604_800,
        _ => return None,
    };
    Some(number * unit).filter(|seconds| (1..=28 * 86_400).contains(seconds))
}

/// What the built-in `command` does with `draft`.
pub fn run_built_in(command: &AppCommand, draft: &str) -> Result<BuiltIn, String> {
    let args = draft
        .strip_prefix('/')
        .and_then(|rest| rest.strip_prefix(command.name.as_str()))
        .unwrap_or("");
    let filled = filled(args, &command.options)?;
    let get = |name: &str| -> Option<Value> {
        filled.iter().find(|option| option["name"] == name).map(|option| option["value"].clone())
    };
    let text = |name: &str| get(name).and_then(|value| value.as_str().map(str::to_owned)).unwrap_or_default();
    let join = |face: &str| {
        let message = text("message");
        if message.is_empty() { face.to_owned() } else { format!("{message} {face}") }
    };
    Ok(match command.name.as_str() {
        "shrug" => BuiltIn::Text(join("\u{af}\\\\_(\u{30c4})\\_/\u{af}")),
        "tableflip" => BuiltIn::Text(join("(\u{256f}\u{b0}\u{25a1}\u{b0})\u{256f}\u{fe35} \u{253b}\u{2501}\u{253b}")),
        "unflip" => BuiltIn::Text(join("\u{252c}\u{2500}\u{252c}\u{30ce}( \u{ba} _ \u{ba}\u{30ce})")),
        "lenny" => BuiltIn::Text(join("( \u{361}\u{b0} \u{35c}\u{296} \u{361}\u{b0})")),
        "me" => BuiltIn::Text(format!("_{}_", text("message"))),
        "spoiler" => BuiltIn::Text(format!("||{}||", text("message"))),
        "nick" => BuiltIn::Nick(text("new_nick")),
        "leave" => BuiltIn::Leave,
        "kick" => BuiltIn::Kick { user: text("user"), reason: text("reason") },
        "ban" => BuiltIn::Ban {
            user: text("user"),
            delete_seconds: get("delete_messages")
                .and_then(|hours| hours.as_u64())
                .map_or(0, |hours| (hours.min(168) * 3600) as u32),
            reason: text("reason"),
        },
        "timeout" => BuiltIn::Timeout {
            user: text("user"),
            seconds: seconds(&text("duration")).ok_or("duration: use 60s, 5m, 10m, 1h, 1d or 1w (up to 28 days)")?,
            reason: text("reason"),
        },
        _ => return Err("Unknown command".into()),
    })
}

/// The command `draft` names, among `commands`, if it starts with one.
pub fn named<'a>(draft: &str, commands: &'a [AppCommand]) -> Option<&'a AppCommand> {
    let rest = draft.strip_prefix('/')?;
    let name = rest.split(char::is_whitespace).next()?;
    commands.iter().find(|command| command.name == name)
}

/// One line of the command menu: a command, or one of its subcommands, as
/// Discord lists them (`invites mcpvp`).
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub label: String,
    pub description: String,
}

/// A command's lines in the command menu.
pub fn entries(command: &AppCommand) -> Vec<Entry> {
    fn walk(prefix: &str, description: &str, options: &[CommandOption], out: &mut Vec<Entry>) {
        let subs: Vec<&CommandOption> = options
            .iter()
            .filter(|option| option.kind == SUBCOMMAND || option.kind == GROUP)
            .collect();
        if subs.is_empty() {
            out.push(Entry { label: prefix.to_owned(), description: description.to_owned() });
            return;
        }
        for sub in subs {
            walk(&format!("{prefix} {}", sub.name), &sub.description, &sub.options, out);
        }
    }
    let mut out = Vec::new();
    walk(&command.name, &command.description, &command.options, &mut out);
    out
}

/// One of the options a command being typed takes, as its bar shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct HintOption {
    pub name: String,
    pub required: bool,
    /// Something is typed for it.
    pub filled: bool,
    /// The one being typed now.
    pub current: bool,
}

/// What the bar over the composer shows while a command is typed: the
/// command, its options, what the one being typed is for and its choices.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Hint {
    pub command: String,
    pub options: Vec<HintOption>,
    pub description: String,
    pub choices: Vec<String>,
}

/// Where `key:` starts in `args` as a word of its own.
fn key_at(args: &str, name: &str) -> Option<usize> {
    let key = format!("{name}:");
    let mut from = 0;
    while let Some(at) = args[from..].find(&key) {
        let at = from + at;
        if at == 0 || args[..at].ends_with(char::is_whitespace) {
            return Some(at);
        }
        from = at + key.len();
    }
    None
}

/// The bar for `draft`, once a command's name is typed and followed by a space.
pub fn hint(draft: &str, commands: &[AppCommand]) -> Option<Hint> {
    let command = named(draft, commands)?;
    let mut rest = draft.strip_prefix('/')?.strip_prefix(command.name.as_str())?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let mut label = command.name.clone();
    let mut options = &command.options;
    loop {
        let subs: Vec<&CommandOption> = options
            .iter()
            .filter(|option| option.kind == SUBCOMMAND || option.kind == GROUP)
            .collect();
        if subs.is_empty() {
            break;
        }
        let trimmed = rest.trim_start();
        let (word, after) = trimmed.split_once(char::is_whitespace).unwrap_or((trimmed, ""));
        match subs.iter().find(|sub| sub.name == word) {
            Some(sub) if trimmed.len() > word.len() => {
                label = format!("{label} {}", sub.name);
                options = &sub.options;
                rest = after;
                if !trimmed[word.len()..].starts_with(char::is_whitespace) {
                    break;
                }
            }
            _ => {
                return Some(Hint {
                    command: label,
                    options: Vec::new(),
                    description: "Pick one".into(),
                    choices: subs.iter().map(|sub| sub.name.clone()).collect(),
                });
            }
        }
    }
    let args = rest;
    let last = options
        .iter()
        .filter_map(|option| Some((key_at(args, &option.name)?, option)))
        .max_by_key(|(at, _)| *at)
        .map(|(_, option)| option);
    let current = last.or_else(|| {
        if args.trim().is_empty() {
            options.iter().find(|option| option.required).or(options.first())
        } else {
            options.first()
        }
    });
    let positional = last.is_none() && !args.trim().is_empty();
    Some(Hint {
        command: label,
        options: options
            .iter()
            .map(|option| HintOption {
                name: option.name.clone(),
                required: option.required,
                filled: key_at(args, &option.name).is_some()
                    || (positional && options.first().is_some_and(|first| first.name == option.name)),
                current: current.is_some_and(|current| current.name == option.name),
            })
            .collect(),
        description: current.map(|option| option.description.clone()).unwrap_or_default(),
        choices: current
            .map(|option| {
                if !option.choices.is_empty() {
                    option.choices.iter().map(|(name, _)| name.clone()).collect()
                } else if option.kind == 5 {
                    vec!["True".into(), "False".into()]
                } else {
                    Vec::new()
                }
            })
            .unwrap_or_default(),
    })
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
    // A command a bot registered for one server names it, as Discord's client
    // sends it; without it Discord answers "Unknown Integration".
    if let Some(server) = command.raw["guild_id"].as_str() {
        body["data"]["guild_id"] = json!(server);
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
        let lines: Vec<String> = commands.iter().flat_map(entries).map(|entry| entry.label).collect();
        assert_eq!(lines, ["leave", "loop song", "loop queue", "play"]);
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
    fn built_ins_run() {
        let list = built_ins(true);
        let run = |draft: &str| run_built_in(named(draft, &list).unwrap(), draft);
        assert_eq!(run("/me waves").unwrap(), BuiltIn::Text("_waves_".into()));
        assert_eq!(run("/spoiler secret").unwrap(), BuiltIn::Text("||secret||".into()));
        assert!(matches!(run("/shrug ok").unwrap(), BuiltIn::Text(text) if text.starts_with("ok ")));
        assert_eq!(
            run("/timeout user:<@5> duration:10m reason:spam").unwrap(),
            BuiltIn::Timeout { user: "5".into(), seconds: 600, reason: "spam".into() }
        );
        assert!(run("/timeout user:<@5> duration:soon").is_err());
        assert!(named("/leave", &list).is_none());
        let dm = built_ins(false);
        assert_eq!(run_built_in(named("/leave", &dm).unwrap(), "/leave silent:True").unwrap(), BuiltIn::Leave);
    }

    #[test]
    fn hints() {
        let commands = parse_index(&index());
        assert!(hint("/play", &commands).is_none());
        let shown = hint("/play ", &commands).unwrap();
        assert_eq!(shown.command, "play");
        assert!(shown.options[0].current && !shown.options[0].filled);
        let shown = hint("/play query:abc next:", &commands).unwrap();
        assert!(shown.options[0].filled && shown.options[1].current);
        assert_eq!(shown.choices, ["True", "False"]);
        let shown = hint("/loop ", &commands).unwrap();
        assert_eq!(shown.choices, ["song", "queue"]);
        let shown = hint("/loop queue ", &commands).unwrap();
        assert_eq!(shown.command, "loop queue");
        assert_eq!(shown.options[0].name, "times");
    }
}
