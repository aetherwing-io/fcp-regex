use std::sync::Arc;

use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::*,
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::domain::model::{FragmentRegistry, RegexEvent};
use crate::domain::mutation;
use crate::domain::query;
use crate::fcpcore::event_log::EventLog;
use crate::fcpcore::parsed_op::parse_op;
use crate::reference_card::REFERENCE_CARD;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MutationParams {
    /// Regex fragment operation strings, e.g. 'define digits any:digit+', 'compile semver anchored:true'
    pub ops: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryParams {
    /// Read-only query string, e.g. 'show digits', 'list library', 'test semver against:1.2.3'
    pub q: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SessionParams {
    /// Session action: 'new "Title" [flavor:pcre]', 'close', 'status', 'undo', 'redo', 'checkpoint NAME'
    pub action: String,
}

struct RegexState {
    registry: FragmentRegistry,
    event_log: EventLog<RegexEvent>,
    title: String,
    flavor: String,
    active: bool,
}

#[derive(Clone)]
pub struct RegexServer {
    state: Arc<Mutex<RegexState>>,
    tool_router: ToolRouter<Self>,
}

impl Default for RegexServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl RegexServer {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RegexState {
                registry: FragmentRegistry::new(),
                event_log: EventLog::new(),
                title: String::new(),
                flavor: "pcre".to_string(),
                active: false,
            })),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Execute regex fragment operations. Each op: VERB [args...] [key:value ...]. Verbs: define, from, compile, drop, rename. Example: 'define digits any:digit+', 'compile semver anchored:true'")]
    async fn regex(
        &self,
        Parameters(p): Parameters<MutationParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut state = self.state.lock().await;
        let mut results = Vec::new();

        for op_str in &p.ops {
            let op = match parse_op(op_str) {
                Ok(o) => o,
                Err(e) => {
                    results.push(format!("ERROR: {}", e.error));
                    continue;
                }
            };

            let (msg, event) = match op.verb.as_str() {
                "define" => mutation::handle_define(&op, &mut state.registry),
                "from" => mutation::handle_from(&op, &mut state.registry),
                "compile" => mutation::handle_compile(&op, &state.registry),
                "drop" => mutation::handle_drop(&op, &mut state.registry),
                "rename" => mutation::handle_rename(&op, &mut state.registry),
                _ => (format!("ERROR: unknown verb {:?}", op.verb), None),
            };

            if let Some(ev) = event {
                state.event_log.append(ev);
            }
            results.push(msg);
        }

        // Append digest
        let digest = format!(
            "[{} fragments]",
            state.registry.len()
        );
        results.push(digest);

        Ok(CallToolResult::success(vec![Content::text(
            results.join("\n"),
        )]))
    }

    #[tool(description = "Execute a read-only query on the regex session. Examples: 'show digits', 'test semver against:1.2.3', 'list', 'list library', 'get semver', 'map'")]
    async fn regex_query(
        &self,
        Parameters(p): Parameters<QueryParams>,
    ) -> Result<CallToolResult, McpError> {
        let state = self.state.lock().await;
        let result = query::handle_query(&p.q, &state.registry);
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Manage the regex session. Actions: 'new \"Title\" [flavor:pcre]', 'close', 'status', 'undo', 'redo', 'checkpoint NAME'")]
    async fn regex_session(
        &self,
        Parameters(p): Parameters<SessionParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self.handle_session(&p.action).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Show the FCP regex reference card with all available verbs and their syntax.")]
    async fn regex_help(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(
            REFERENCE_CARD,
        )]))
    }
}

impl RegexServer {
    async fn handle_session(&self, action: &str) -> String {
        let tokens: Vec<&str> = action.split_whitespace().collect();
        if tokens.is_empty() {
            return "! empty session action".to_string();
        }

        match tokens[0] {
            "new" => self.handle_new(&tokens).await,
            "close" => self.handle_close().await,
            "status" => self.handle_status().await,
            "undo" => self.handle_undo().await,
            "redo" => self.handle_redo().await,
            "checkpoint" => self.handle_checkpoint(&tokens).await,
            _ => format!("! unknown session action {:?}", tokens[0]),
        }
    }

    async fn handle_new(&self, tokens: &[&str]) -> String {
        let mut state = self.state.lock().await;

        // Parse title (may be quoted)
        let title = if tokens.len() > 1 {
            let rest = tokens[1..].join(" ");
            // Extract quoted title if present
            if let Some(start) = rest.find('"') {
                if let Some(end) = rest[start + 1..].find('"') {
                    rest[start + 1..start + 1 + end].to_string()
                } else {
                    rest[start + 1..].to_string()
                }
            } else {
                tokens[1].to_string()
            }
        } else {
            "Untitled".to_string()
        };

        // Parse flavor param
        let flavor = tokens
            .iter()
            .find_map(|t| t.strip_prefix("flavor:"))
            .unwrap_or("pcre")
            .to_string();

        state.registry = FragmentRegistry::new();
        state.event_log = EventLog::new();
        state.title = title.clone();
        state.flavor = flavor.clone();
        state.active = true;

        format!("+ new session {:?} (flavor: {})", title, flavor)
    }

    async fn handle_close(&self) -> String {
        let mut state = self.state.lock().await;
        state.registry = FragmentRegistry::new();
        state.event_log = EventLog::new();
        state.title.clear();
        state.active = false;
        "- session closed".to_string()
    }

    async fn handle_status(&self) -> String {
        let state = self.state.lock().await;
        if state.active {
            format!(
                "= session: {:?}, flavor: {}, fragments: {}, can_undo: {}, can_redo: {}",
                state.title,
                state.flavor,
                state.registry.len(),
                state.event_log.can_undo(),
                state.event_log.can_redo(),
            )
        } else {
            "= no active session".to_string()
        }
    }

    async fn handle_undo(&self) -> String {
        let mut state = self.state.lock().await;
        let events = state.event_log.undo(1);
        if events.is_empty() {
            return "! nothing to undo".to_string();
        }
        for event in &events {
            reverse_event(event, &mut state.registry);
        }
        format!("= undone {} event(s)", events.len())
    }

    async fn handle_redo(&self) -> String {
        let mut state = self.state.lock().await;
        let events = state.event_log.redo(1);
        if events.is_empty() {
            return "! nothing to redo".to_string();
        }
        for event in &events {
            replay_event(event, &mut state.registry);
        }
        format!("= redone {} event(s)", events.len())
    }

    async fn handle_checkpoint(&self, tokens: &[&str]) -> String {
        if tokens.len() < 2 {
            return "! checkpoint requires a name".to_string();
        }
        let name = tokens[1];
        let mut state = self.state.lock().await;
        state.event_log.checkpoint(name);
        format!("= checkpoint {:?} created", name)
    }
}

fn reverse_event(event: &RegexEvent, registry: &mut FragmentRegistry) {
    match event {
        RegexEvent::Define { name, old, .. } => {
            if let Some(old_elements) = old {
                let _ = registry.define(name, old_elements.clone());
            } else {
                let _ = registry.drop(name);
            }
        }
        RegexEvent::Drop { name, elements } => {
            let _ = registry.define(name, elements.clone());
        }
        RegexEvent::Rename { old_name, new_name } => {
            let _ = registry.rename(new_name, old_name);
        }
    }
}

fn replay_event(event: &RegexEvent, registry: &mut FragmentRegistry) {
    match event {
        RegexEvent::Define { name, new, .. } => {
            let _ = registry.define(name, new.clone());
        }
        RegexEvent::Drop { name, .. } => {
            let _ = registry.drop(name);
        }
        RegexEvent::Rename { old_name, new_name } => {
            let _ = registry.rename(old_name, new_name);
        }
    }
}

#[tool_handler]
impl ServerHandler for RegexServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "FCP Regex server for building regular expressions via named fragment composition. \
                 Use regex_session to start a session, regex for mutations, regex_query for queries, \
                 and regex_help for the reference card."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_text(result: &CallToolResult) -> String {
        result
            .content
            .iter()
            .filter_map(|c| match &c.raw {
                RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    #[tokio::test]
    async fn test_regex_define_and_compile() {
        let server = RegexServer::new();
        let params = MutationParams {
            ops: vec![
                "define digits any:digit+".to_string(),
                "compile digits".to_string(),
            ],
        };
        let result = server.regex(Parameters(params)).await.unwrap();
        let text = get_text(&result);
        assert!(text.contains("fragment"));
        assert!(text.contains(r"\d+"));
    }

    #[tokio::test]
    async fn test_regex_unknown_verb() {
        let server = RegexServer::new();
        let params = MutationParams {
            ops: vec!["explode everything".to_string()],
        };
        let result = server.regex(Parameters(params)).await.unwrap();
        let text = get_text(&result);
        assert!(text.contains("unknown verb"));
    }

    #[tokio::test]
    async fn test_regex_parse_error() {
        let server = RegexServer::new();
        let params = MutationParams {
            ops: vec!["".to_string()],
        };
        let result = server.regex(Parameters(params)).await.unwrap();
        let text = get_text(&result);
        assert!(text.contains("ERROR"));
    }

    #[tokio::test]
    async fn test_regex_query_dispatches() {
        let server = RegexServer::new();
        let params = QueryParams {
            q: "list".to_string(),
        };
        let result = server.regex_query(Parameters(params)).await.unwrap();
        let text = get_text(&result);
        assert!(text.contains("no fragments"));
    }

    #[tokio::test]
    async fn test_regex_session_new() {
        let server = RegexServer::new();
        let params = SessionParams {
            action: "new \"My Regex\" flavor:pcre".to_string(),
        };
        let result = server.regex_session(Parameters(params)).await.unwrap();
        let text = get_text(&result);
        assert!(text.contains("My Regex"));
        assert!(text.contains("pcre"));
    }

    #[tokio::test]
    async fn test_regex_session_status_inactive() {
        let server = RegexServer::new();
        let params = SessionParams {
            action: "status".to_string(),
        };
        let result = server.regex_session(Parameters(params)).await.unwrap();
        let text = get_text(&result);
        assert!(text.contains("no active session"));
    }

    #[tokio::test]
    async fn test_regex_session_close() {
        let server = RegexServer::new();
        // Start session
        server
            .regex_session(Parameters(SessionParams {
                action: "new Test".to_string(),
            }))
            .await
            .unwrap();
        // Close
        let result = server
            .regex_session(Parameters(SessionParams {
                action: "close".to_string(),
            }))
            .await
            .unwrap();
        let text = get_text(&result);
        assert!(text.contains("closed"));
    }

    #[tokio::test]
    async fn test_regex_session_undo_redo() {
        let server = RegexServer::new();
        // Define something
        server
            .regex(Parameters(MutationParams {
                ops: vec!["define digits any:digit+".to_string()],
            }))
            .await
            .unwrap();
        // Undo
        let result = server
            .regex_session(Parameters(SessionParams {
                action: "undo".to_string(),
            }))
            .await
            .unwrap();
        let text = get_text(&result);
        assert!(text.contains("undone"));
        // Redo
        let result = server
            .regex_session(Parameters(SessionParams {
                action: "redo".to_string(),
            }))
            .await
            .unwrap();
        let text = get_text(&result);
        assert!(text.contains("redone"));
    }

    #[tokio::test]
    async fn test_regex_session_empty_action() {
        let server = RegexServer::new();
        let params = SessionParams {
            action: "".to_string(),
        };
        let result = server.regex_session(Parameters(params)).await.unwrap();
        let text = get_text(&result);
        assert!(text.contains("empty"));
    }

    #[tokio::test]
    async fn test_regex_session_checkpoint() {
        let server = RegexServer::new();
        let params = SessionParams {
            action: "checkpoint v1".to_string(),
        };
        let result = server.regex_session(Parameters(params)).await.unwrap();
        let text = get_text(&result);
        assert!(text.contains("checkpoint"));
        assert!(text.contains("v1"));
    }

    #[tokio::test]
    async fn test_regex_help() {
        let server = RegexServer::new();
        let result = server.regex_help().await.unwrap();
        let text = get_text(&result);
        assert!(text.contains("define"));
        assert!(text.contains("compile"));
        assert!(text.contains("FRAGMENTS"));
    }

    #[tokio::test]
    async fn test_server_info() {
        let server = RegexServer::new();
        let info = server.get_info();
        assert!(info.instructions.is_some());
        let instructions = info.instructions.unwrap();
        assert!(instructions.contains("FCP Regex"));
    }

    #[tokio::test]
    async fn test_digest_appended() {
        let server = RegexServer::new();
        let params = MutationParams {
            ops: vec!["define digits any:digit+".to_string()],
        };
        let result = server.regex(Parameters(params)).await.unwrap();
        let text = get_text(&result);
        assert!(text.contains("[1 fragments]"));
    }
}
