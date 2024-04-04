use std::fs::{self, OpenOptions};
use std::panic::PanicInfo;
use std::path::PathBuf;

use dashmap::DashMap;
use kotlin::KotlinFile;
use tower_lsp::jsonrpc::{Error, ErrorCode, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing::{info, warn};
use tree_sitter::Parser;
use walkdir::WalkDir;

mod hover;
pub mod kotlin;

struct Backend {
    client: Client,
    files: DashMap<PathBuf, KotlinFile>,
}

impl Backend {
    fn new(client: Client) -> Backend {
        Backend {
            client,
            files: DashMap::new(),
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
            let content = fs::read(&path).unwrap();
            let tree = parser.parse(&content, None).unwrap();
            match KotlinFile::new(&tree, &content) {
                Ok(f) => {
                    self.files.insert(path, f);
                }
                Err(err) => {
                    warn!("{path:?}: {:?}", err);
                }
            }
        }

        info!("parsed {} kotlin files", self.files.len());

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

        let path = params
            .text_document_position_params
            .text_document
            .uri
            .to_file_path()
            .map_err(|_| Error {
                code: ErrorCode::InternalError,
                message: "invalid path".into(),
                data: None,
            })?;

        let pos = params.text_document_position_params.position;
        self.get_hover(&path, &pos).map_err(|err| Error {
            code: ErrorCode::InternalError,
            message: err.to_string().into(),
            data: None,
        })
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

    let (service, socket) = LspService::new(Backend::new);
    info!("starting server");
    Server::new(stdin, stdout, socket).serve(service).await;
}
