use dashmap::DashMap;
use ropey::Rope;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

mod completion;
mod diagnostics;
mod hover;
mod symbols;

#[derive(Debug)]
struct Document {
    rope: Rope,
}

impl Document {
    fn new(content: String) -> Self {
        let rope = Rope::from_str(&content);
        Self { rope }
    }
}

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<DashMap<Url, RwLock<Document>>>,
    symbol_table: Arc<RwLock<HashMap<String, SymbolInfo>>>,
}

#[derive(Debug, Clone)]
struct SymbolInfo {
    name: String,
    kind: SymbolKind,
    location: Location,
    documentation: Option<String>,
    parameters: Option<Vec<String>>,
    return_type: Option<String>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        tracing::info!("SKILL LSP server initializing...");

        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
                    ..Default::default()
                },
            )),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(true),
                trigger_characters: Some(vec![
                    "(".to_string(),
                    " ".to_string(),
                    "-".to_string(),
                ]),
                ..Default::default()
            }),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            references_provider: Some(OneOf::Left(true)),
            signature_help_provider: Some(SignatureHelpOptions {
                trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                ..Default::default()
            }),
            document_formatting_provider: Some(OneOf::Left(true)),
            rename_provider: Some(OneOf::Left(true)),
            code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: vec!["skill.addDocComment".to_string()],
                ..Default::default()
            }),
            ..Default::default()
        };

        Ok(InitializeResult {
            capabilities,
            server_info: Some(ServerInfo {
                name: "SKILL LSP".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("SKILL LSP server initialized!");
        self.client
            .log_message(MessageType::INFO, "SKILL LSP server is ready")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("SKILL LSP server shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let content = params.text_document.text;
        let doc = Document::new(content.clone());

        // Index symbols in the document
        let symbols = symbols::extract_symbols(&content, &uri);
        let mut symbol_table = self.symbol_table.write().await;
        for symbol in symbols {
            symbol_table.insert(symbol.name.clone(), symbol);
        }

        self.documents.insert(uri.clone(), RwLock::new(doc));

        // Publish diagnostics
        self.publish_diagnostics(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        if let Some(content_change) = params.content_changes.first() {
            if let Some(doc_mut) = self.documents.get(&uri) {
                let mut doc = doc_mut.write().await;
                doc.rope = Rope::from_str(&content_change.text);

                // Re-index symbols
                let content = content_change.text.clone();
                let symbols = symbols::extract_symbols(&content, &uri);
                let mut symbol_table = self.symbol_table.write().await;
                // Clear old symbols for this document and re-insert
                symbol_table.retain(|_, v| v.location.uri != uri);
                for symbol in symbols {
                    symbol_table.insert(symbol.name.clone(), symbol);
                }
            }
        }

        self.publish_diagnostics(&uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        self.publish_diagnostics(&uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let items = completion::get_completions(&self.documents, &self.symbol_table, &uri, position)
            .await;

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn completion_resolve(&self, item: CompletionItem) -> Result<CompletionItem> {
        Ok(item)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let hover_info = hover::get_hover(&self.documents, &self.symbol_table, &uri, position).await;

        Ok(hover_info)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let location = symbols::goto_definition(&self.documents, &self.symbol_table, &uri, position)
            .await;

        Ok(location.map(GotoDefinitionResponse::Scalar))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        let symbols = symbols::get_document_symbols(&self.symbol_table, &uri).await;

        Ok(Some(DocumentSymbolResponse::Flat(symbols)))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let locations = symbols::find_references(&self.documents, &self.symbol_table, &uri, position)
            .await;

        Ok(Some(locations))
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let help = completion::get_signature_help(&self.documents, &self.symbol_table, &uri, position)
            .await;

        Ok(help)
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;

        let edits = diagnostics::format_document(&self.documents, &uri).await;

        Ok(Some(edits))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let range = params.range;

        let mut actions = Vec::new();

        // Add documentation comment action
        if let Some(doc) = self.documents.get(&uri) {
            let doc = doc.read().await;
            let line = doc.rope.line(range.start.line as usize);
            let text = line.to_string();

            if text.trim().starts_with("(defun") || text.trim().starts_with("(procedure") {
                let action = CodeActionOrCommand::CodeAction(CodeAction {
                    title: "Add documentation comment".to_string(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    command: Some(Command {
                        title: "Add doc comment".to_string(),
                        command: "skill.addDocComment".to_string(),
                        arguments: Some(vec![
                            serde_json::to_value(&uri).unwrap(),
                            serde_json::to_value(&range.start.line).unwrap(),
                        ]),
                    }),
                    ..Default::default()
                });
                actions.push(action);
            }
        }

        Ok(Some(actions))
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<serde_json::Value>> {
        match params.command.as_str() {
            "skill.addDocComment" => {
                let args = params.arguments;
                if let (Some(uri_val), Some(line_val)) = (args.first(), args.get(1)) {
                    let uri: Url = serde_json::from_value(uri_val.clone()).unwrap();
                    let line: u32 = serde_json::from_value(line_val.clone()).unwrap();
                    self.add_documentation_comment(&uri, line).await;
                }
            }
            _ => {}
        }
        Ok(None)
    }
}

impl Backend {
    async fn publish_diagnostics(&self, uri: &Url) {
        let diagnostics = diagnostics::get_diagnostics(&self.documents, uri).await;

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }

    async fn add_documentation_comment(&self, uri: &Url, line: u32) {
        if let Some(doc_mut) = self.documents.get(uri) {
            let doc = doc_mut.read().await;
            let func_line = doc.rope.line(line as usize);
            let func_text = func_line.to_string();

            // Extract function name and parameters
            let re = regex::Regex::new(r"\((?:defun|procedure)\s+(\w+)\s*\(([^)]*)\)").unwrap();
            if let Some(caps) = re.captures(&func_text) {
                let func_name = caps.get(1).unwrap().as_str();
                let params = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                let doc_comment = if params.is_empty() {
                    format!(
                        ";;;\n;;; @function {}\n;;; @description \n;;; @return\n",
                        func_name
                    )
                } else {
                    let param_list: Vec<&str> = params.split_whitespace().collect();
                    let mut comment = format!(
                        ";;;\n;;; @function {}\n;;; @description \n",
                        func_name
                    );
                    for param in &param_list {
                        comment.push_str(&format!(";;; @param {} \n", param));
                    }
                    comment.push_str(";;; @return\n");
                    comment
                };

                let edit = TextEdit {
                    range: Range {
                        start: Position {
                            line,
                            character: 0,
                        },
                        end: Position {
                            line,
                            character: 0,
                        },
                    },
                    new_text: format!("{}\n", doc_comment),
                };

                let workspace_edit = WorkspaceEdit {
                    changes: Some({
                        let mut map = HashMap::new();
                        map.insert(uri.clone(), vec![edit]);
                        map
                    }),
                    ..Default::default()
                };

                self.client
                    .apply_edit(workspace_edit)
                    .await
                    .ok();
            }
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: Arc::new(DashMap::new()),
        symbol_table: Arc::new(RwLock::new(HashMap::new())),
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
