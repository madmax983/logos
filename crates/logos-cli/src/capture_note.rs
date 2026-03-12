use chrono::DateTime;
use serde::Deserialize;

/// Parsed capture-note payload sourced from one Markdown file in the vault inbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureNote {
    pub capture_id: String,
    pub captured_at: String,
    pub kind: String,
    pub amount_cents: i64,
    pub currency: String,
    pub merchant_memo: String,
    pub from_account_hint: Option<String>,
    pub to_account_hint: Option<String>,
    pub category_hint: Option<String>,
    pub status: String,
    pub source: Option<String>,
    pub device_id: Option<String>,
    pub body: String,
}

/// Parser failures for Markdown capture notes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureParseError {
    MissingFrontmatter,
    UnterminatedFrontmatter,
    InvalidFrontmatter(String),
    InvalidField {
        field: &'static str,
        message: String,
    },
}

impl std::fmt::Display for CaptureParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingFrontmatter => write!(f, "capture note is missing YAML frontmatter"),
            Self::UnterminatedFrontmatter => {
                write!(f, "capture note frontmatter is missing a closing delimiter")
            }
            Self::InvalidFrontmatter(message) => {
                write!(f, "invalid capture note frontmatter: {message}")
            }
            Self::InvalidField { field, message } => {
                write!(f, "invalid capture field '{field}': {message}")
            }
        }
    }
}

impl std::error::Error for CaptureParseError {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaptureNoteFrontmatter {
    capture_id: String,
    captured_at: String,
    kind: String,
    amount_cents: i64,
    #[serde(default = "default_currency")]
    currency: String,
    merchant_memo: String,
    from_account_hint: Option<String>,
    to_account_hint: Option<String>,
    category_hint: Option<String>,
    #[serde(default = "default_status")]
    status: String,
    source: Option<String>,
    device_id: Option<String>,
}

impl CaptureNote {
    /// Parses one Markdown capture note with YAML frontmatter.
    ///
    /// # Errors
    ///
    /// Returns an error when the note is missing frontmatter, the YAML is invalid,
    /// or required fields fail validation.
    pub fn parse(raw: &str) -> Result<Self, CaptureParseError> {
        let (frontmatter_raw, body_raw) = split_frontmatter(raw)?;
        let frontmatter = serde_yaml::from_str::<CaptureNoteFrontmatter>(&frontmatter_raw)
            .map_err(|err| CaptureParseError::InvalidFrontmatter(err.to_string()))?;

        let capture_id = require_trimmed(frontmatter.capture_id, "capture_id")?;
        let captured_at = require_trimmed(frontmatter.captured_at, "captured_at")?;
        DateTime::parse_from_rfc3339(&captured_at).map_err(|err| {
            CaptureParseError::InvalidField {
                field: "captured_at",
                message: err.to_string(),
            }
        })?;

        let kind = require_trimmed(frontmatter.kind, "kind")?;
        if !matches!(kind.as_str(), "expense" | "income" | "transfer" | "cash") {
            return Err(CaptureParseError::InvalidField {
                field: "kind",
                message: format!("expected expense|income|transfer|cash, got '{kind}'"),
            });
        }

        let currency = require_trimmed(frontmatter.currency, "currency")?;
        let merchant_memo = require_trimmed(frontmatter.merchant_memo, "merchant_memo")?;
        let status = require_trimmed(frontmatter.status, "status")?;

        Ok(Self {
            capture_id,
            captured_at,
            kind,
            amount_cents: frontmatter.amount_cents,
            currency,
            merchant_memo,
            from_account_hint: normalize_optional(frontmatter.from_account_hint),
            to_account_hint: normalize_optional(frontmatter.to_account_hint),
            category_hint: normalize_optional(frontmatter.category_hint),
            status,
            source: normalize_optional(frontmatter.source),
            device_id: normalize_optional(frontmatter.device_id),
            body: body_raw,
        })
    }
}

fn split_frontmatter(raw: &str) -> Result<(String, String), CaptureParseError> {
    let mut lines = raw.lines();
    let Some(first_line) = lines.next() else {
        return Err(CaptureParseError::MissingFrontmatter);
    };
    if first_line.trim_end() != "---" {
        return Err(CaptureParseError::MissingFrontmatter);
    }

    let mut frontmatter_lines = Vec::new();
    let mut body_lines = Vec::new();
    let mut in_body = false;

    for line in lines {
        if !in_body && line.trim_end() == "---" {
            in_body = true;
            continue;
        }

        if in_body {
            body_lines.push(line);
        } else {
            frontmatter_lines.push(line);
        }
    }

    if !in_body {
        return Err(CaptureParseError::UnterminatedFrontmatter);
    }

    Ok((frontmatter_lines.join("\n"), body_lines.join("\n")))
}

fn require_trimmed(value: String, field: &'static str) -> Result<String, CaptureParseError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CaptureParseError::InvalidField {
            field,
            message: "must not be empty".to_owned(),
        });
    }
    Ok(trimmed.to_owned())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    })
}

fn default_currency() -> String {
    "USD".to_owned()
}

fn default_status() -> String {
    "inbox".to_owned()
}
