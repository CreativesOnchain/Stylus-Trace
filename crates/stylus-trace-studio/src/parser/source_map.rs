//! Source mapping service for Stylus WASM binaries.
//!
//! # ⚠️ CURRENTLY NON-FUNCTIONAL
//! This service translates binary offsets (PCs) to source locations (file:line) using DWARF.
//! However, it is currently non-functional because the Arbitrum `stylusTracer` does not
//! provide the required Program Counter (PC) offsets for WASM execution.
//!
//! This code is preserved for future use when tracer support is improved.

//! Source mapping service for Stylus WASM binaries.
//!
//! Translates binary offsets (PCs) to source locations (file:line) using DWARF.

use addr2line::Context;
use log::{debug, info};
use std::path::Path;

/// A location in the source code
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub function: Option<String>,
}

type Reader = addr2line::gimli::EndianReader<addr2line::gimli::RunTimeEndian, std::rc::Rc<[u8]>>;

/// Mapper that handles address translation
pub struct SourceMapper {
    context: Option<Context<Reader>>,
}

impl SourceMapper {
    /// Create a new SourceMapper from a WASM file
    pub fn new<P: AsRef<Path>>(wasm_path: P) -> anyhow::Result<Self> {
        let path = wasm_path.as_ref();
        debug!("Loading WASM binary for source mapping: {}", path.display());

        let file_data = std::fs::read(path)?;
        let obj = object::File::parse(&*file_data)?;

        let context = Context::new(&obj).ok();

        if context.is_none() {
            info!("No debug information (DWARF) found in the WASM binary. Source-to-line mapping will not be available.");
            info!("Tip: Compile your contract with `debug = true` in your Cargo.toml release profile.");
        } else {
            info!("Debug information loaded successfully. Source-to-line mapping enabled.");
        }

        Ok(Self { context })
    }

    /// Factory for an empty mapper (fallback)
    pub fn empty() -> Self {
        Self { context: None }
    }

    /// Lookup source location for a given offset
    pub fn lookup(&self, offset: u64) -> Option<SourceLocation> {
        let context = self.context.as_ref()?;

        // In addr2line 0.21, find_frames returns a LookupResult.
        // For synchronous use with all data loaded, we can use skip_all_loads().
        let mut frames = context.find_frames(offset).skip_all_loads().ok()?;

        if let Some(frame) = frames.next().ok().flatten() {
            let function = frame
                .function
                .and_then(|f| f.demangle().ok().map(|d| d.into_owned()));

            let location = frame.location;

            return Some(SourceLocation {
                file: location
                    .as_ref()
                    .and_then(|l| l.file)
                    .map(|f| f.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                line: location.as_ref().and_then(|l| l.line),
                column: location.as_ref().and_then(|l| l.column),
                function,
            });
        }

        None
    }
}
