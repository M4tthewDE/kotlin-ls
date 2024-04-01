use std::fs::{self, OpenOptions};
use std::path::PathBuf;

use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing::{info, warn};
use tree_sitter::{Parser, Tree};
use walkdir::WalkDir;

struct Backend {
    client: Client,
    trees: DashMap<PathBuf, Tree>,
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
            let tree = parser.parse(content, None).unwrap();
            self.trees.insert(path, tree);
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
        let tree = self
            .trees
            .get(
                &params
                    .text_document_position_params
                    .text_document
                    .uri
                    .to_file_path()
                    .unwrap(),
            )
            .unwrap();

        let pos = params.text_document_position_params.position;

        let mut cursor = tree.walk();
        let mut target_node = None;
        'outer: loop {
            let node = cursor.node();
            if node.start_position().row <= pos.line as usize
                && node.start_position().column <= pos.character as usize
                && node.end_position().row >= pos.line as usize
                && node.end_position().column >= pos.character as usize
            {
                target_node = Some(node);
            }

            if cursor.goto_first_child() {
                continue;
            }

            loop {
                if cursor.goto_next_sibling() {
                    break;
                }

                if !cursor.goto_parent() {
                    break 'outer;
                }
            }
        }

        let target_node = target_node.unwrap();

        info!(
            "kind: {} | start: {} | end: {}",
            target_node.kind(),
            target_node.start_position(),
            target_node.end_position(),
        );

        let hover = Hover {
            contents: HoverContents::Scalar(MarkedString::String("hello hover".to_string())),
            range: None,
        };

        Ok(Some(hover))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
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
