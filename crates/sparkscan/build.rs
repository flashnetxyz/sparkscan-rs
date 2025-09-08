cfg_if::cfg_if! {
    if #[cfg(feature = "tracing")] {
        use syn::{ItemImpl, ItemStruct, parse_quote, visit_mut::VisitMut, ItemMod};
    } else {
        use syn::{ItemImpl, parse_quote, visit_mut::VisitMut};
    }
}

use schemars::schema::{InstanceType, SchemaObject};

/// Read documentation from a markdown file and convert it to doc attributes
fn read_doc_from_file(filename: &str) -> Vec<syn::Attribute> {
    let doc_path = std::path::Path::new("doc").join(filename);
    let content = std::fs::read_to_string(&doc_path)
        .unwrap_or_else(|_| panic!("Failed to read documentation file: {:?}", doc_path));

    content
        .lines()
        .map(|line| {
            syn::parse_quote! { #[doc = #line] }
        })
        .collect()
}

/// Get documentation filename for a method name
fn get_doc_filename_for_method(method_name: &str) -> Option<&'static str> {
    match method_name {
        // Client methods
        "new" => Some("new.md"),
        "new_with_client" => Some("new_with_client.md"),
        "new_with_api_key" => Some("new_with_api_key.md"),

        // Root endpoint
        "root_get" => Some("root_get.md"),

        // Address endpoints
        "address_summary_v1_address_address_get" => Some("address_summary.md"),
        "get_address_transactions_v1_address_address_transactions_get" => {
            Some("get_address_transactions.md")
        }
        "get_address_tokens_v1_address_address_tokens_get" => Some("get_address_tokens.md"),

        // Transaction endpoints
        "get_latest_transactions_v1_tx_latest_get" => Some("get_latest_transactions.md"),

        // Stats endpoints
        "get_wallet_leaderboard_v1_stats_leaderboard_wallets_get" => {
            Some("get_wallet_leaderboard.md")
        }
        "get_token_leaderboard_v1_stats_leaderboard_tokens_get" => Some("get_token_leaderboard.md"),

        // Token endpoints
        "get_token_transactions_v1_tokens_identifier_transactions_get" => {
            Some("get_token_transactions.md")
        }
        "get_token_holders_v1_tokens_identifier_holders_get" => Some("get_token_holders.md"),
        "token_issuer_lookup_v1_tokens_issuer_lookup_post" => Some("token_issuer_lookup.md"),

        // Bitcoin endpoints
        "get_addresses_latest_txid_v1_bitcoin_addresses_latest_txid_post" => {
            Some("get_addresses_latest_txid.md")
        }

        _ => None,
    }
}

fn main() {
    let src = "./openapi.json";
    println!("cargo:rerun-if-changed={}", src);
    println!("cargo:rerun-if-changed=doc/");

    let file = std::fs::File::open(src).unwrap();
    let spec = serde_json::from_reader(file).unwrap();

    let mut settings = progenitor::GenerationSettings::new();
    settings.with_interface(progenitor::InterfaceStyle::Builder);

    // Replace all integer schemas with i128
    settings.with_conversion(
        SchemaObject {
            instance_type: Some(InstanceType::Integer.into()),
            ..Default::default()
        },
        "i128",
        std::iter::empty(),
    );

    // Replace all number schemas with f64
    settings.with_conversion(
        SchemaObject {
            instance_type: Some(InstanceType::Number.into()),
            ..Default::default()
        },
        "f64",
        std::iter::empty(),
    );

    // Replace number schemas with float format specifically with f64
    settings.with_conversion(
        SchemaObject {
            instance_type: Some(InstanceType::Number.into()),
            format: Some("float".to_string()),
            ..Default::default()
        },
        "f64",
        std::iter::empty(),
    );

    let mut generator = progenitor::Generator::new(&settings);
    let tokens = generator.generate_tokens(&spec).unwrap();
    let mut ast = syn::parse2(tokens).unwrap();

    let mut import_modifier = ImportModifier::new();
    import_modifier.visit_file_mut(&mut ast);

    let mut headers_modifier = ClientHeadersModifier::new();
    headers_modifier.visit_file_mut(&mut ast);

    let mut doc_modifier = ClientDocumentationModifier::new();
    doc_modifier.visit_file_mut(&mut ast);

    let mut untagged_i128_injector = UntaggedI128Injector;
    untagged_i128_injector.visit_file_mut(&mut ast);

    #[cfg(feature = "tracing")]
    {
        let mut tracing_modifier = ClientTracingModifier;
        tracing_modifier.visit_file_mut(&mut ast);

        let mut builder_instrumenter = BuilderSendInstrumenter::new();
        builder_instrumenter.visit_file_mut(&mut ast);
    }

    // Generate the code first
    let mut content = prettyplease::unparse(&ast);

    // Inject the custom i128 deserializer function inside the types module
    let i128_deserializer = r#"
    // Custom deserializer for i128 values in untagged enums
    fn deserialize_i128<'de, D>(des: D) -> Result<i128, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct I128Visitor;
        impl<'de> serde::de::Visitor<'de> for I128Visitor {
            type Value = i128;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "an integer or a string representing an integer")
            }

            fn visit_i64<E>(self, v: i64) -> Result<i128, E> {
                Ok(v as i128)
            }

            fn visit_u64<E>(self, v: u64) -> Result<i128, E> {
                Ok(v as i128)
            }

            fn visit_str<E>(self, v: &str) -> Result<i128, E>
            where
                E: serde::de::Error,
            {
                v.parse::<i128>().map_err(|e| serde::de::Error::custom(format!("invalid i128 string: {}", e)))
            }

            fn visit_string<E>(self, v: String) -> Result<i128, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&v)
            }
        }

        des.deserialize_any(I128Visitor)
    }

    // Custom deserializer for Option<i128> values in untagged enums
    fn deserialize_option_i128<'de, D>(des: D) -> Result<Option<i128>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct OptionI128Visitor;
        impl<'de> serde::de::Visitor<'de> for OptionI128Visitor {
            type Value = Option<i128>;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "an integer, a string representing an integer, or null")
            }

            fn visit_none<E>(self) -> Result<Option<i128>, E> {
                Ok(None)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Option<i128>, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserialize_i128(deserializer).map(Some)
            }

            fn visit_unit<E>(self) -> Result<Option<i128>, E> {
                Ok(None)
            }

            fn visit_i64<E>(self, v: i64) -> Result<Option<i128>, E> {
                Ok(Some(v as i128))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Option<i128>, E> {
                Ok(Some(v as i128))
            }

            fn visit_str<E>(self, v: &str) -> Result<Option<i128>, E>
            where
                E: serde::de::Error,
            {
                v.parse::<i128>().map(Some).map_err(|e| serde::de::Error::custom(format!("invalid i128 string: {}", e)))
            }

            fn visit_string<E>(self, v: String) -> Result<Option<i128>, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&v)
            }
        }

        des.deserialize_any(OptionI128Visitor)
    }
"#;

    // Insert the deserializer function inside the types module
    if let Some(types_start) = content.find("pub mod types {") {
        if let Some(insertion_point) = content[types_start..].find("/// Error types.") {
            let full_insertion_point = types_start + insertion_point;
            content.insert_str(full_insertion_point, i128_deserializer);
        }
    }
    let out_file = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("codegen.rs");
    std::fs::write(out_file, content).unwrap();
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

struct ClientHeadersModifier {
    modified: bool,
}

impl ClientHeadersModifier {
    fn new() -> Self {
        Self { modified: false }
    }
}

impl syn::visit_mut::VisitMut for ClientHeadersModifier {
    fn visit_item_impl_mut(&mut self, item: &mut ItemImpl) {
        let is_client_impl = matches!(&item.self_ty.as_ref(),
            syn::Type::Path(p) if p.path.is_ident("Client"));

        if is_client_impl && item.trait_.is_none() {
            for impl_item in &mut item.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    if method.sig.ident.to_string().as_str() == "new" {
                        method.block = parse_quote! {{
                            Self::new_with_client(baseurl, Self::base_client())
                        }};
                    }
                }
            }

            let has_new_method = item
                .items
                .iter()
                .any(|item| matches!(item, syn::ImplItem::Fn(method) if method.sig.ident == "new"));

            if has_new_method {
                // Add new method to the impl block
                let get_base_url_method: syn::ImplItem = parse_quote! {
                    /// Get the base URL of the client
                    fn base_client() -> reqwest::Client {
                        let user_agent = format!("sparkscan-rs/{}", env!("CARGO_PKG_VERSION"));
                        let mut headers = reqwest::header::HeaderMap::new();
                        headers.insert(
                            reqwest::header::USER_AGENT,
                            user_agent.parse().unwrap(),
                        );

                        #[cfg(not(target_arch = "wasm32"))]
                        let client = {
                            let dur = std::time::Duration::from_secs(15);
                            reqwest::ClientBuilder::new()
                                .connect_timeout(dur)
                                .timeout(dur)
                                .default_headers(headers)
                                .build()
                                .unwrap()
                        };
                        #[cfg(target_arch = "wasm32")]
                        let client = reqwest::ClientBuilder::new()
                            .default_headers(headers)
                            .build()
                            .unwrap();

                        client
                    }
                };

                // Add new_with_api_key method
                let new_with_api_key_method: syn::ImplItem = parse_quote! {
                    /// Create a new client with an API key for production use with api.sparkscan.io
                    pub fn new_with_api_key(baseurl: &str, api_key: &str) -> Self {
                        let user_agent = format!("sparkscan-rs/{}", env!("CARGO_PKG_VERSION"));
                        let mut headers = reqwest::header::HeaderMap::new();
                        headers.insert(
                            reqwest::header::USER_AGENT,
                            user_agent.parse().unwrap(),
                        );
                        headers.insert(
                            "x-api-key",
                            api_key.parse().unwrap(),
                        );

                        #[cfg(not(target_arch = "wasm32"))]
                        let client = {
                            let dur = std::time::Duration::from_secs(15);
                            reqwest::ClientBuilder::new()
                                .connect_timeout(dur)
                                .timeout(dur)
                                .default_headers(headers)
                                .build()
                                .unwrap()
                        };
                        #[cfg(target_arch = "wasm32")]
                        let client = reqwest::ClientBuilder::new()
                            .default_headers(headers)
                            .build()
                            .unwrap();

                        Self::new_with_client(baseurl, client)
                    }
                };

                item.items.push(get_base_url_method);
                item.items.push(new_with_api_key_method);
                self.modified = true;
            }
        }

        syn::visit_mut::visit_item_impl_mut(self, item);
    }
}

struct ClientDocumentationModifier {
    modified: bool,
}

impl ClientDocumentationModifier {
    fn new() -> Self {
        Self { modified: false }
    }
}

impl syn::visit_mut::VisitMut for ClientDocumentationModifier {
    fn visit_item_impl_mut(&mut self, item: &mut ItemImpl) {
        let is_client_impl = matches!(&item.self_ty.as_ref(),
            syn::Type::Path(p) if p.path.is_ident("Client"));

        if is_client_impl && item.trait_.is_none() {
            for impl_item in &mut item.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    let method_name = method.sig.ident.to_string();

                    // Apply documentation from external files
                    if let Some(doc_filename) = get_doc_filename_for_method(&method_name) {
                        // Remove existing documentation attributes (but preserve other attributes)
                        method.attrs.retain(|attr| !attr.path().is_ident("doc"));

                        // Add documentation from file
                        let doc_attrs = read_doc_from_file(doc_filename);
                        method.attrs.extend(doc_attrs);
                        self.modified = true;
                    }
                }
            }
        }

        syn::visit_mut::visit_item_impl_mut(self, item);
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
                        "new_with_api_key" => {
                            method.block = parse_quote! {{
                                let user_agent = format!("sparkscan-rs/{}", env!("CARGO_PKG_VERSION"));
                                let mut headers = reqwest::header::HeaderMap::new();
                                headers.insert(
                                    reqwest::header::USER_AGENT,
                                    user_agent.parse().unwrap(),
                                );
                                headers.insert(
                                    "x-api-key",
                                    api_key.parse().unwrap(),
                                );

                                #[cfg(not(target_arch = "wasm32"))]
                                let client = {
                                    let dur = std::time::Duration::from_secs(15);
                                    reqwest::ClientBuilder::new()
                                        .connect_timeout(dur)
                                        .timeout(dur)
                                        .default_headers(headers)
                                        .build()
                                        .unwrap()
                                };
                                #[cfg(target_arch = "wasm32")]
                                let client = reqwest::ClientBuilder::new()
                                    .default_headers(headers)
                                    .build()
                                    .unwrap();

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

struct UntaggedI128Injector;

impl syn::visit_mut::VisitMut for UntaggedI128Injector {
    fn visit_item_struct_mut(&mut self, item: &mut syn::ItemStruct) {
        // Apply to ALL structs that have i128 fields, not just specific ones
        if let syn::Fields::Named(fields) = &mut item.fields {
            for field in &mut fields.named {
                if let Some(field_name) = &field.ident {
                    // Check if this field is i128 (direct or Option<i128>)
                    let needs_custom_deserializer = match &field.ty {
                        syn::Type::Path(type_path) => {
                            // Check for direct i128
                            if type_path.path.segments.len() == 1 {
                                type_path.path.segments[0].ident == "i128"
                            } else {
                                let last_segment = type_path.path.segments.last().unwrap();
                                if last_segment.ident == "i128" {
                                    true
                                } else if last_segment.ident == "Option" {
                                    // Check if it's Option<i128>
                                    if let syn::PathArguments::AngleBracketed(args) =
                                        &last_segment.arguments
                                    {
                                        if let Some(syn::GenericArgument::Type(syn::Type::Path(
                                            inner_path,
                                        ))) = args.args.first()
                                        {
                                            inner_path.path.segments.last().unwrap().ident == "i128"
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            }
                        }
                        _ => false,
                    };

                    if needs_custom_deserializer {
                        // Check if it already has a deserialize_with attribute
                        let already_has_custom = field.attrs.iter().any(|attr| {
                            attr.path().is_ident("serde")
                                && format!("{:?}", attr).contains("deserialize_with")
                        });

                        if !already_has_custom {
                            // Determine which deserializer to use
                            let deserializer_name = if let syn::Type::Path(type_path) = &field.ty {
                                let last_segment = type_path.path.segments.last().unwrap();
                                if last_segment.ident == "Option" {
                                    "deserialize_option_i128"
                                } else {
                                    "deserialize_i128"
                                }
                            } else {
                                "deserialize_i128"
                            };

                            // Add the appropriate custom deserializer attribute
                            field.attrs.push(parse_quote! {
                                #[serde(deserialize_with = #deserializer_name)]
                            });

                            println!(
                                "cargo:warning=Added custom {} deserializer to {}.{}",
                                deserializer_name, item.ident, field_name
                            );
                        }
                    }
                }
            }
        }

        syn::visit_mut::visit_item_struct_mut(self, item);
    }
}
