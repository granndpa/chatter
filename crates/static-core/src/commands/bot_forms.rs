//! Bot modal parsing, validation and the type-5 submission payload.
use serde_json::{Value, json};

#[derive(Clone, Debug)]
pub struct Field {
    pub label: String,
    pub placeholder: String,
    pub value: String,
    pub multiline: bool,
    pub required: bool,
    pub min: usize,
    pub max: usize,
    path: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct BotForm {
    pub id: String,
    pub title: String,
    pub fields: Vec<Field>,
    pub busy: bool,
    pub error: Option<String>,
    context: Value,
    custom_id: String,
    components: Value,
}

impl BotForm {
    pub fn parse(modal: &Value, context: Value) -> Result<Self, String> {
        let text = |key: &str| {
            modal[key]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "Incomplete bot form".to_owned())
        };
        let mut form = Self {
            id: text("id")?,
            title: text("title")?,
            custom_id: text("custom_id")?,
            components: modal["components"].clone(),
            context,
            fields: Vec::new(),
            busy: false,
            error: None,
        };
        let rows = form.components.as_array().ok_or("Incomplete bot form")?;
        if rows.is_empty() || rows.len() > 5 {
            return Err("Unsupported bot form layout".into());
        }
        for (index, row) in rows.iter().enumerate() {
            // Legacy action rows and modern label wrappers both contain inputs.
            let (input, path, label) = match row["type"].as_u64() {
                Some(1) if row["components"].as_array().is_some_and(|c| c.len() == 1) => (
                    &row["components"][0],
                    vec![index, 0],
                    row["components"][0]["label"].as_str(),
                ),
                Some(18) => (&row["component"], vec![index], row["label"].as_str()),
                _ => return Err("This bot form uses an unsupported field".into()),
            };
            if input["type"] != 4 || input["custom_id"].as_str().is_none() {
                return Err("This bot form uses an unsupported field".into());
            }
            form.fields.push(Field {
                label: label.unwrap_or("Answer").to_owned(),
                placeholder: input["placeholder"].as_str().unwrap_or_default().to_owned(),
                value: input["value"].as_str().unwrap_or_default().to_owned(),
                multiline: input["style"] == 2,
                required: input["required"].as_bool().unwrap_or(true),
                min: input["min_length"].as_u64().unwrap_or(0).min(4000) as usize,
                max: input["max_length"].as_u64().unwrap_or(4000).min(4000) as usize,
                path,
            });
        }
        Ok(form)
    }

    pub fn submit(&self, nonce: &str) -> Result<Value, String> {
        let mut components = self.components.clone();
        for field in &self.fields {
            let length = field.value.encode_utf16().count();
            if field.required && length == 0 {
                return Err(format!("{} is required", field.label));
            }
            if (length > 0 && length < field.min) || length > field.max {
                return Err(format!(
                    "{} must contain {} to {} characters",
                    field.label, field.min, field.max
                ));
            }
            let row = &mut components[field.path[0]];
            let input = if field.path.len() == 2 {
                &mut row["components"][0]
            } else {
                &mut row["component"]
            };
            *input = json!({"type": 4, "custom_id": input["custom_id"], "value": field.value});
            // Labels, styling and defaults are presentation, not submitted data.
            if let Some(object) = row.as_object_mut() {
                object.remove("label");
                object.remove("description");
            }
        }
        let mut body = json!({
            "type": 5, "nonce": nonce,
            "application_id": self.context["application_id"],
            "channel_id": self.context["channel_id"],
            "data": {"id": self.id, "custom_id": self.custom_id, "components": components},
        });
        if let Some(guild) = self.context.get("guild_id") {
            body["guild_id"] = guild.clone();
        }
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_form_retains_context_and_validates_before_submitting() {
        let modal = json!({"id":"100", "custom_id":"ticket", "title":"Open ticket", "components":[
            {"type":1,"components":[{"type":4,"custom_id":"subject","style":1,"label":"Subject","min_length":2,"max_length":4}]},
            {"type":18,"label":"Details","component":{"type":4,"custom_id":"details","style":2,"required":false}}
        ]});
        let mut form = BotForm::parse(
            &modal,
            json!({"application_id":"9","channel_id":"8","guild_id":"7"}),
        )
        .unwrap();
        assert!(form.submit("2").unwrap_err().contains("required"));
        form.fields[0].value = "a".into();
        assert!(form.submit("2").is_err());
        form.fields[0].value = "Test".into();
        form.fields[1].value = "First\nSecond".into();
        let body = form.submit("2").unwrap();
        assert_eq!(body["type"], 5);
        assert_eq!(body["data"]["id"], "100");
        assert_eq!(body["guild_id"], "7");
        assert_eq!(
            body["data"]["components"][0]["components"][0],
            json!({"type":4,"custom_id":"subject","value":"Test"})
        );
        assert_eq!(
            body["data"]["components"][1]["component"]["value"],
            "First\nSecond"
        );
        assert!(body["data"]["components"][1].get("label").is_none());
        form.fields[0].value = "12345".into();
        assert!(form.submit("2").is_err());
    }

    #[test]
    fn refuses_unsupported_fields_instead_of_sending_incomplete_answers() {
        assert!(BotForm::parse(&json!({"id":"1","custom_id":"x","title":"x","components":[{"type":1,"components":[{"type":3,"custom_id":"x"}]}]}), json!({})).is_err());
    }
}
