cfg_if::cfg_if! {
    if #[cfg(feature = "tracing")] {
        use syn::{ItemImpl, ItemStruct, parse_quote, visit_mut::VisitMut};
    } else {
        use syn::{ItemImpl, parse_quote, visit_mut::VisitMut};
    }
}

fn main() {
    let src = "./sparkscan.json";
    println!("cargo:rerun-if-changed={}", src);

    let file = std::fs::File::open(src).unwrap();
    let spec = serde_json::from_reader(file).unwrap();

    let mut settings = progenitor::GenerationSettings::default();
    settings.with_interface(progenitor::InterfaceStyle::Builder);

    let mut generator = progenitor::Generator::new(&settings);
    let tokens = generator.generate_tokens(&spec).unwrap();
    let mut ast = syn::parse2(tokens).unwrap();

    let mut headers_modifier = ClientHeadersModifier::new();
    headers_modifier.visit_file_mut(&mut ast);
    // Only apply modifications if tracing feature is enabled
    #[cfg(feature = "tracing")]
    {
        let mut tracing_modifier = ClientTracingModifier;
        tracing_modifier.visit_file_mut(&mut ast);
    }

    let content = prettyplease::unparse(&ast);
    let out_file = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("codegen.rs");
    std::fs::write(out_file, content).unwrap();
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
                    match method.sig.ident.to_string().as_str() {
                        "new" => {
                            method.block = parse_quote! {{
                                Self::new_with_client(baseurl, Self::base_client())
                            }};
                        }
                        _ => {}
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
                item.items.push(get_base_url_method);
                self.modified = true;
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
                        method.block = parse_quote! {{
                            unimplemented!("client() method is not supported when using middleware")
                        }};
                    }
                }
            }
        }

        syn::visit_mut::visit_item_impl_mut(self, item);
    }
}
