use std::fs::{self, OpenOptions};
use std::panic::PanicInfo;
use std::path::PathBuf;

use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing::{info, warn};
use tree_sitter::{Parser, Tree};
use walkdir::WalkDir;

mod tree;

struct Backend {
    client: Client,
    trees: DashMap<PathBuf, (Tree, String)>,
}

impl Backend {
    fn new(client: Client) -> Backend {
        Backend {
            client,
            trees: DashMap::new(),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        info!("general: {:?}", params.capabilities);
        info!("client-info: {:?}", params.client_info);
        info!("root-uri: {:?}", params.root_uri);

        let mut parser = Parser::new();
        parser.set_language(tree_sitter_kotlin::language()).unwrap();
        for path in WalkDir::new(params.root_uri.unwrap().path())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "kt"))
            .map(|e| e.into_path())
        {
            let content = fs::read_to_string(&path).unwrap();
            let tree = parser.parse(&content, None).unwrap();
            self.trees.insert(path, (tree, content));
        }

        info!("parsed {} trees", self.trees.len());

        let capas = ServerCapabilities {
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            ..Default::default()
        };

        let res = InitializeResult {
            capabilities: capas,
            ..Default::default()
        };

        Ok(res)
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn did_open(&self, _: DidOpenTextDocumentParams) {
        warn!("Got a textDocument/didOpen notification, but it is not implemented");
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        info!("file saved: {:?}", params);
        self.client
            .log_message(MessageType::INFO, format!("file saved: {:?}", params))
            .await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        info!("hover: {:?}", params);
        let (tree, content) = self
            .trees
            .get(
                &params
                    .text_document_position_params
                    .text_document
                    .uri
                    .to_file_path()
                    .unwrap(),
            )
            .unwrap()
            .to_owned();

        let pos = params.text_document_position_params.position;
        let target_node = tree::get_node(&tree, &pos).unwrap();

        info!(
            "[target_node] kind: {} | code: {} | start: {} | end: {}",
            target_node.kind(),
            target_node.utf8_text(&content.as_bytes()).unwrap(),
            target_node.start_position(),
            target_node.end_position(),
        );

        let parent = target_node.parent().unwrap();
        let hover = match parent.kind() {
            "call_expression" => {
                let name = target_node.utf8_text(&content.as_bytes()).unwrap();
                let function = tree::get_function(&tree, &content, name).unwrap();

                Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("```kotlin\n{function}\n```"),
                    }),
                    range: None,
                }
            }
            _ => Hover {
                contents: HoverContents::Scalar(MarkedString::String(format!(
                    "{} is not supported yet",
                    parent.kind()
                ))),
                range: None,
            },
        };

        Ok(Some(hover))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

pub fn panic_hook(panic_info: &PanicInfo) {
    let payload = panic_info.payload();

    #[allow(clippy::manual_map)]
    let payload = if let Some(s) = payload.downcast_ref::<&str>() {
        Some(&**s)
    } else if let Some(s) = payload.downcast_ref::<String>() {
        Some(s.as_str())
    } else {
        None
    };

    let location = panic_info.location().map(|l| l.to_string());

    tracing::error!(
        panic.payload = payload,
        panic.location = location,
        "A panic occurred",
    );
}

#[tokio::main]
async fn main() {
    let _ = std::panic::catch_unwind(|| {
        std::panic::set_hook(Box::new(panic_hook));
        panic!("This is a static panic message");
    });
    let log_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(std::env::var("KOTLIN_LS_LOG").unwrap())
        .unwrap();
    tracing_subscriber::fmt().with_writer(log_file).init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
