use std::fs::{self, File, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};

use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing::{info, warn};
use tree_sitter::{Parser, Tree};
use walkdir::WalkDir;

mod kotlin;

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

        // for debugging
        // let f = File::create("/home/matti/Programming/kotlin-ls/graph.dot").unwrap();
        // tree.print_dot_graph(&f.as_raw_fd());

        let mut cursor = tree.walk();
        loop {
            let node = cursor.node();
            info!(
                "node kind {}, start: {}, end: {}",
                node.kind(),
                node.start_position(),
                node.end_position(),
            );

            if cursor.goto_first_child() {
                continue;
            }

            if cursor.goto_next_sibling() {
                continue;
            }

            while !cursor.goto_parent() || !cursor.goto_next_sibling() {
                if !cursor.goto_parent() {
                    break;
                }
            }

            if cursor.node() == tree.root_node() {
                break;
            }
        }

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
