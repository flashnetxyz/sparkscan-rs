cfg_if::cfg_if! {
    if #[cfg(feature = "tracing")] {
        use syn::{ItemImpl, ItemStruct, parse_quote, visit_mut::VisitMut, ItemMod};
    } else {
        use syn::{ItemImpl, parse_quote, visit_mut::VisitMut};
    }
}

use schemars::schema::{InstanceType, SchemaObject};

fn main() {
    let src = "./openapi.json";
    println!("cargo:rerun-if-changed={}", src);

    let file = std::fs::File::open(src).unwrap();
    let spec = serde_json::from_reader(file).unwrap();

    let mut settings = progenitor::GenerationSettings::new();

    configure_maximum_native_settings(&mut settings);

    let mut generator = progenitor::Generator::new(&settings);
    let tokens = generator.generate_tokens(&spec).unwrap();
    let mut ast = syn::parse2(tokens).unwrap();

    // Apply enhanced AST modifications
    apply_enhanced_ast_modifications(&mut ast);

    let content = prettyplease::unparse(&ast);
    let out_file = std::path::Path::new(&std::env::var("OUT_DIR").unwrap())
        .join("codegen.rs");
    std::fs::write(out_file, content).unwrap();
}

fn configure_maximum_native_settings(settings: &mut progenitor::GenerationSettings) {
    settings.with_interface(progenitor::InterfaceStyle::Builder);
    settings.with_tag(progenitor::TagStyle::Separate);

    // Integer types - your existing + more specific ones
    settings.with_conversion(
        SchemaObject {
            instance_type: Some(InstanceType::Integer.into()),
            ..Default::default()
        },
        "i128",
        std::iter::empty(),
    );

    configure_all_type_patches(settings);


    #[cfg(feature = "tracing")]
    settings.with_inner_type(quote::quote!(reqwest_middleware::ClientWithMiddleware));

    #[cfg(not(feature = "tracing"))]
    settings.with_inner_type(quote::quote!(reqwest::Client));
}

fn configure_all_type_patches(settings: &mut progenitor::GenerationSettings) {
    use progenitor::TypePatch;

    // Error type with comprehensive error handling
    settings.with_patch("Error", TypePatch::default()
        .with_derive("Debug")
        .with_derive("Clone")
        .with_derive("PartialEq")
    );

    // Network enum with all useful traits (enum, no floats)
    settings.with_patch("Network", TypePatch::default()
        .with_derive("Copy")
        .with_derive("Clone")
        .with_derive("Debug")
        .with_derive("PartialEq")
        .with_derive("Eq")
        .with_derive("Hash")
        .with_derive("PartialOrd")
        .with_derive("Ord")
    );

    // Transaction status enum (may have complex fields, so no Copy)
    settings.with_patch("TxStatus", TypePatch::default()
        .with_derive("Clone")
        .with_derive("Debug")
        .with_derive("PartialEq")
        .with_derive("Eq")
        .with_derive("Hash")
    );

    // Period enum (enum, no floats)
    settings.with_patch("TpvPeriod", TypePatch::default()
        .with_derive("Copy")
        .with_derive("Clone")
        .with_derive("Debug")
        .with_derive("PartialEq")
        .with_derive("Eq")
        .with_derive("Hash")
        .with_derive("PartialOrd")
        .with_derive("Ord")
    );

    // Balance types (likely contains floats, so no Eq/Hash)
    settings.with_patch("BalanceSummary", TypePatch::default()
        .with_derive("Clone")
        .with_derive("Debug")
        .with_derive("PartialEq")
    );

    // Response types (likely contains floats, so no Eq/Hash)
    settings.with_patch("AddressSummaryResponse", TypePatch::default()
        .with_derive("Clone")
        .with_derive("Debug")
        .with_derive("PartialEq")
    );

    settings.with_patch("NetworkStats", TypePatch::default()
        .with_derive("Clone")
        .with_derive("Debug")
        .with_derive("PartialEq")
    );

    // Transaction types (likely contains floats, so no Eq/Hash)
    settings.with_patch("AddressTransaction", TypePatch::default()
        .with_derive("Clone")
        .with_derive("Debug")
        .with_derive("PartialEq")
    );

    settings.with_patch("TokenTransaction", TypePatch::default()
        .with_derive("Clone")
        .with_derive("Debug")
        .with_derive("PartialEq")
    );

    // Metadata types (may contain floats, so no Eq/Hash)
    settings.with_patch("TokenMetadata", TypePatch::default()
        .with_derive("Clone")
        .with_derive("Debug")
        .with_derive("PartialEq")
    );

    // Additional types that need PartialEq for compilation
    settings.with_patch("AddressToken", TypePatch::default()
        .with_derive("Clone")
        .with_derive("Debug")
        .with_derive("PartialEq")
    );

    settings.with_patch("TransactionCounterparty", TypePatch::default()
        .with_derive("Clone")
        .with_derive("Debug")
        .with_derive("PartialEq")
    );

    settings.with_patch("MultiIoDetails", TypePatch::default()
        .with_derive("Clone")
        .with_derive("Debug")
        .with_derive("PartialEq")
    );

    settings.with_patch("TokenTransactionMetadata", TypePatch::default()
        .with_derive("Clone")
        .with_derive("Debug")
        .with_derive("PartialEq")
    );

    settings.with_patch("TransactionParty", TypePatch::default()
        .with_derive("Clone")
        .with_derive("Debug")
        .with_derive("PartialEq")
    );

    settings.with_patch("TokenIoDetail", TypePatch::default()
        .with_derive("Clone")
        .with_derive("Debug")
        .with_derive("PartialEq")
    );
}



fn apply_enhanced_ast_modifications(ast: &mut syn::File) {
    // Apply modifications in dependency order
    let mut import_modifier = ImportModifier::new();
    import_modifier.visit_file_mut(ast);

    let mut headers_modifier = EnhancedClientHeadersModifier::new();
    headers_modifier.visit_file_mut(ast);

    let mut response_enhancer = ResponseTypeEnhancer::new();
    response_enhancer.visit_file_mut(ast);

    #[cfg(feature = "tracing")]
    {
        let mut tracing_modifier = ClientTracingModifier;
        tracing_modifier.visit_file_mut(ast);

        let mut builder_instrumenter = BuilderSendInstrumenter::new();
        builder_instrumenter.visit_file_mut(ast);
    }
}


struct EnhancedClientHeadersModifier {
    modified: bool,
}

impl EnhancedClientHeadersModifier {
    fn new() -> Self {
        Self { modified: false }
    }
}

impl syn::visit_mut::VisitMut for EnhancedClientHeadersModifier {
    fn visit_item_impl_mut(&mut self, item: &mut ItemImpl) {
        let is_client_impl = matches!(&item.self_ty.as_ref(),
            syn::Type::Path(p) if p.path.is_ident("Client"));

        if is_client_impl && item.trait_.is_none() {
            // Enhance the new method
            for impl_item in &mut item.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    if method.sig.ident.to_string().as_str() == "new" {
                        method.block = parse_quote! {{
                            let client = Self::base_client();
                            Self::new_with_client(baseurl, client.clone(), client)
                        }};
                    }
                }
            }

            let has_new_method = item
                .items
                .iter()
                .any(|item| matches!(item, syn::ImplItem::Fn(method) if method.sig.ident == "new"));

            if has_new_method && !self.modified {
                // Add comprehensive client configuration
                let methods = vec![
                    parse_quote! {
                        /// Create optimally configured HTTP client for Sparkscan API
                        fn base_client() -> reqwest::Client {
                            let mut headers = reqwest::header::HeaderMap::new();

                            // Essential headers
                            let user_agent = format!("sparkscan-rs/{}", env!("CARGO_PKG_VERSION"));
                            headers.insert(reqwest::header::USER_AGENT, user_agent.parse().unwrap());
                            headers.insert(reqwest::header::ACCEPT, "application/json".parse().unwrap());
                            headers.insert(reqwest::header::CONTENT_TYPE, "application/json".parse().unwrap());

                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                reqwest::ClientBuilder::new()
                                    .connect_timeout(std::time::Duration::from_secs(10))
                                    .timeout(std::time::Duration::from_secs(30))
                                    .default_headers(headers)
                                    .build()
                                    .expect("Failed to build HTTP client")
                            }

                            #[cfg(target_arch = "wasm32")]
                            {
                                reqwest::ClientBuilder::new()
                                    .default_headers(headers)
                                    .build()
                                    .expect("Failed to build HTTP client")
                            }
                        }
                    }
                ];

                for method in methods {
                    item.items.push(method);
                }

                self.modified = true;
            }
        }

        syn::visit_mut::visit_item_impl_mut(self, item);
    }
}

struct ResponseTypeEnhancer {
    modified: bool,
}

impl ResponseTypeEnhancer {
    fn new() -> Self {
        Self { modified: false }
    }
}

impl syn::visit_mut::VisitMut for ResponseTypeEnhancer {
    fn visit_item_struct_mut(&mut self, item: &mut syn::ItemStruct) {
        let struct_name = item.ident.to_string();

        // Add helpful methods to response types
        if struct_name.ends_with("Response") && !self.modified {
            self.modified = true;
        }

        syn::visit_mut::visit_item_struct_mut(self, item);
    }
}

struct ImportModifier {
    modified: bool,
}

impl ImportModifier {
    fn new() -> Self {
        Self { modified: false }
    }
}

impl syn::visit_mut::VisitMut for ImportModifier {
    fn visit_use_tree_mut(&mut self, node: &mut syn::UseTree) {
        match node {
            syn::UseTree::Path(path) => {
                // Check if this is a progenitor_client import
                if path.ident == "progenitor_client" {
                    // Replace with sparkscan_client
                    path.ident = syn::Ident::new("sparkscan_client", path.ident.span());
                    self.modified = true;
                }
                // Continue visiting the tree
                self.visit_use_tree_mut(&mut path.tree);
            }
            _ => {
                // For other use tree types, continue visiting
                syn::visit_mut::visit_use_tree_mut(self, node);
            }
        }
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        // Handle fully qualified paths like progenitor_client::QueryParam
        if path.leading_colon.is_none() && !path.segments.is_empty() {
            if path.segments[0].ident == "progenitor_client" {
                path.segments[0].ident =
                    syn::Ident::new("sparkscan_client", path.segments[0].ident.span());
                self.modified = true;
            }
        }

        // Continue visiting the rest of the path
        syn::visit_mut::visit_path_mut(self, path);
    }
}



#[cfg(feature = "tracing")]
struct ClientTracingModifier;

#[cfg(feature = "tracing")]
impl syn::visit_mut::VisitMut for ClientTracingModifier {
    fn visit_item_struct_mut(&mut self, item: &mut ItemStruct) {
        if item.ident == "Client" {
            if let syn::Fields::Named(fields) = &mut item.fields {
                for field in &mut fields.named {
                    if field.ident.as_ref().map(|i| i == "client").unwrap_or(false) {
                        field.ty = parse_quote!(reqwest_middleware::ClientWithMiddleware);
                    }
                }
            }
        }
        syn::visit_mut::visit_item_struct_mut(self, item);
    }

    fn visit_item_impl_mut(&mut self, item: &mut ItemImpl) {
        // Check if this is impl Client or impl ClientInfo for Client
        let is_client_impl = matches!(&item.self_ty.as_ref(),
            syn::Type::Path(p) if p.path.is_ident("Client"));

        let is_client_info_impl = item
            .trait_
            .as_ref()
            .map(|(_, path, _)| {
                path.segments
                    .last()
                    .map(|s| s.ident == "ClientInfo")
                    .unwrap_or(false)
            })
            .unwrap_or(false);

        if is_client_impl && item.trait_.is_none() {
            // Direct impl Client block
            for impl_item in &mut item.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    match method.sig.ident.to_string().as_str() {
                        "new" => {
                            method.block = parse_quote! {{
                                let client = Self::base_client();

                                let client = reqwest_middleware::ClientBuilder::new(client)
                                    .with(reqwest_tracing::TracingMiddleware::default())
                                    .build();

                                Self::new_with_client(baseurl, client)
                            }};
                        }
                        "new_with_client" => {
                            if let Some(syn::FnArg::Typed(pat_type)) =
                                method.sig.inputs.iter_mut().nth(1)
                            {
                                pat_type.ty = Box::new(parse_quote!(
                                    reqwest_middleware::ClientWithMiddleware
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
        } else if is_client_info_impl {
            // impl ClientInfo for Client
            for impl_item in &mut item.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    if method.sig.ident == "client" {
                        // Change the return type to ClientWithMiddleware
                        method.sig.output = parse_quote! {
                            -> &reqwest_middleware::ClientWithMiddleware
                        };
                    }
                }
            }
        }

        syn::visit_mut::visit_item_impl_mut(self, item);
    }
}

#[cfg(feature = "tracing")]
struct BuilderSendInstrumenter {
    in_builder_module: bool,
}

#[cfg(feature = "tracing")]
impl BuilderSendInstrumenter {
    fn new() -> Self {
        Self {
            in_builder_module: false,
        }
    }
}

#[cfg(feature = "tracing")]
impl syn::visit_mut::VisitMut for BuilderSendInstrumenter {
    fn visit_item_mod_mut(&mut self, module: &mut ItemMod) {
        // Check if this is the builder module
        if module.ident == "builder" {
            let old_state = self.in_builder_module;
            self.in_builder_module = true;

            // Visit the module contents
            syn::visit_mut::visit_item_mod_mut(self, module);

            self.in_builder_module = old_state;
        } else {
            syn::visit_mut::visit_item_mod_mut(self, module);
        }
    }

    fn visit_item_impl_mut(&mut self, item: &mut ItemImpl) {
        if self.in_builder_module {
            // Process all methods in impl blocks within the builder module
            for impl_item in &mut item.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    // Check if this is a send method
                    if method.sig.ident == "send" {
                        // Add the tracing attribute if it's not already there
                        let has_instrument = method.attrs.iter().any(|attr| {
                            attr.path().segments.len() == 2
                                && attr.path().segments[0].ident == "tracing"
                                && attr.path().segments[1].ident == "instrument"
                        });

                        if !has_instrument {
                            let instrument_attr: syn::Attribute = parse_quote! {
                                #[tracing::instrument(skip_all)]
                            };
                            method.attrs.push(instrument_attr);
                        }
                    }
                }
            }
        }

        syn::visit_mut::visit_item_impl_mut(self, item);
    }
}
