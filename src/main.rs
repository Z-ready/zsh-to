use std::env;
use std::fmt::{self, Display};
use std::process::{Command, ExitCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Exact,
    Token,
    Path,
    Git,
}

#[derive(Debug, PartialEq, Eq)]
struct QueryArgs {
    db: String,
    mode: Mode,
    terms: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
enum CliError {
    Usage(&'static str),
    UnknownCommand(String),
    UnknownOption(String),
    UnknownMode(String),
    MissingValue(&'static str),
    EmptyQuery,
    SqliteFailed(Option<i32>),
    Io(String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(f, "{message}"),
            Self::UnknownCommand(command) => write!(f, "unknown command: {command}"),
            Self::UnknownOption(option) => write!(f, "unknown option: {option}"),
            Self::UnknownMode(mode) => write!(f, "unknown mode: {mode}"),
            Self::MissingValue(option) => write!(f, "missing value for {option}"),
            Self::EmptyQuery => write!(f, "query requires at least one term"),
            Self::SqliteFailed(code) => write!(f, "sqlite3 failed with status {code:?}"),
            Self::Io(message) => write!(f, "{message}"),
        }
    }
}

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("to-helper: {error}");
            ExitCode::from(1)
        }
    }
}

fn run(args: Vec<String>) -> Result<String, CliError> {
    let (command, rest) = args.split_first().ok_or(CliError::Usage(
        "usage: to-helper query --db <path> --mode <mode> -- <query...>",
    ))?;

    match command.as_str() {
        "query" => query(parse_query_args(rest)?),
        _ => Err(CliError::UnknownCommand(command.to_owned())),
    }
}

fn parse_query_args(args: &[String]) -> Result<QueryArgs, CliError> {
    let mut db: Option<String> = None;
    let mut mode: Option<Mode> = None;
    let mut terms: Vec<String> = Vec::new();
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--db" => {
                index += 1;
                let value = args.get(index).ok_or(CliError::MissingValue("--db"))?;
                db = Some(value.to_owned());
            }
            "--mode" => {
                index += 1;
                let value = args.get(index).ok_or(CliError::MissingValue("--mode"))?;
                mode = Some(parse_mode(value)?);
            }
            "--" => {
                terms.extend(args[(index + 1)..].iter().map(|term| term.to_lowercase()));
                break;
            }
            option if option.starts_with('-') => {
                return Err(CliError::UnknownOption(option.to_owned()));
            }
            term => terms.push(term.to_lowercase()),
        }
        index += 1;
    }

    if terms.is_empty() {
        return Err(CliError::EmptyQuery);
    }

    Ok(QueryArgs {
        db: db.ok_or(CliError::MissingValue("--db"))?,
        mode: mode.ok_or(CliError::MissingValue("--mode"))?,
        terms,
    })
}

fn parse_mode(value: &str) -> Result<Mode, CliError> {
    match value {
        "exact" => Ok(Mode::Exact),
        "token" => Ok(Mode::Token),
        "path" => Ok(Mode::Path),
        "git" => Ok(Mode::Git),
        _ => Err(CliError::UnknownMode(value.to_owned())),
    }
}

fn query(args: QueryArgs) -> Result<String, CliError> {
    let sql = build_query_sql(args.mode, &args.terms)?;
    let output = Command::new("sqlite3")
        .args(["-noheader", &args.db, &sql])
        .output()
        .map_err(|error| CliError::Io(error.to_string()))?;

    if !output.status.success() {
        return Err(CliError::SqliteFailed(output.status.code()));
    }

    String::from_utf8(output.stdout).map_err(|error| CliError::Io(error.to_string()))
}

fn build_query_sql(mode: Mode, terms: &[String]) -> Result<String, CliError> {
    if terms.is_empty() {
        return Err(CliError::EmptyQuery);
    }

    let first = sql_quote(&terms[0]);
    let token_list = terms
        .iter()
        .map(|term| sql_quote(term))
        .collect::<Vec<String>>()
        .join(",");
    let path_where = terms
        .iter()
        .map(|term| format!("lower(path) like {}", sql_quote(&format!("%{term}%"))))
        .collect::<Vec<String>>()
        .join(" and ");

    let order = format!(
        "order by case when {{alias}}lower_name = {first} then 0 else 1 end, \
         {{alias}}last_used desc, {{alias}}hit_count desc, {{alias}}depth asc, \
         length({{alias}}path), {{alias}}path limit 50"
    );

    let sql = match mode {
        Mode::Exact => {
            if terms.len() != 1 {
                return Err(CliError::Usage("exact mode requires one query term"));
            }
            format!(
                "select path from dirs where lower_name = {first} {} ;",
                order.replace("{alias}", "")
            )
        }
        Mode::Token => format!(
            "select d.path from dirs d \
             join tokens t on t.dir_id = d.id \
             where t.token in ({token_list}) \
             group by d.path \
             having count(distinct t.token) = {} {} ;",
            terms.len(),
            order.replace("{alias}", "d.")
        ),
        Mode::Path => format!(
            "select path from dirs where {path_where} {} ;",
            order.replace("{alias}", "")
        ),
        Mode::Git => format!(
            "select d.path from dirs d \
             join tokens t on t.dir_id = d.id \
             where d.is_git = 1 and t.token in ({token_list}) \
             group by d.path \
             having count(distinct t.token) = {} {} ;",
            terms.len(),
            order.replace("{alias}", "d.")
        ),
    };
    Ok(sql)
}

fn sql_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_query_args_when_terms_follow_separator() {
        let args = vec![
            "--db".to_string(),
            "/tmp/index.sqlite3".to_string(),
            "--mode".to_string(),
            "token".to_string(),
            "--".to_string(),
            "App".to_string(),
            "Backend".to_string(),
        ];

        let parsed = parse_query_args(&args).unwrap();

        assert_eq!(
            parsed,
            QueryArgs {
                db: "/tmp/index.sqlite3".to_string(),
                mode: Mode::Token,
                terms: vec!["app".to_string(), "backend".to_string()]
            }
        );
    }

    #[test]
    fn sql_quote_when_value_contains_single_quote() {
        let quoted = sql_quote("kid's");

        assert_eq!(quoted, "'kid''s'");
    }

    #[test]
    fn build_query_sql_when_git_mode_uses_token_join() {
        let sql = build_query_sql(Mode::Git, &["backend".to_string(), "api".to_string()]).unwrap();

        assert!(sql.contains("join tokens"));
        assert!(sql.contains("d.is_git = 1"));
        assert!(sql.contains("having count(distinct t.token) = 2"));
    }
}
