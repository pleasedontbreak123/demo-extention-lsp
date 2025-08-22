use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, DeriveInput, LitStr, Meta, Token,
};

enum MetaOrLitStr {
    Meta(Meta),
    LitStr(LitStr),
}

impl Parse for MetaOrLitStr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // 尝试解析为 Meta
        if let Ok(meta) = input.parse::<Meta>() {
            return Ok(MetaOrLitStr::Meta(meta));
        }
        // 如果不能解析为 Meta，再尝试解析为 LitStr
        if let Ok(lit_str) = input.parse::<LitStr>() {
            return Ok(MetaOrLitStr::LitStr(lit_str));
        }
        Err(input.error("Expected Meta or LitStr"))
    }
}

/// - `#[grammar(".AC", node1, node2)]`
#[proc_macro_derive(TryParse, attributes(grammar, matches))]
pub fn try_parse_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let expanded = match &input.data {
        syn::Data::Struct(data_struct) => {
            let mut codes = Vec::new();
            let mut fields = Vec::new();

            for attr in &input.attrs {
                if attr.path().is_ident("grammar") {
                    let nested = attr
                        .parse_args_with(Punctuated::<MetaOrLitStr, Token![,]>::parse_terminated)
                        .unwrap();

                    for elem in nested {
                        match elem {
                            MetaOrLitStr::Meta(meta) => {
                                match meta {
                                    // #[grammar(node)]
                                    Meta::Path(path) => {
                                        let ident = path.get_ident().unwrap();
                                        codes.push(quote! {
                                          let #ident = self.try_parse()?;
                                        });
                                    }
                                    // #[grammar(op("/", "CSDF"))]
                                    Meta::List(list) => {
                                        let ident = list.path.get_ident().unwrap();
                                        let mut lits = list
                                            .parse_args_with(
                                                Punctuated::<LitStr, Token![,]>::parse_terminated,
                                            )
                                            .unwrap()
                                            .into_iter();

                                        let condition = if let Some(first) = lits.next() {
                                            quote! {  self.matches_consume(#first) }
                                        } else {
                                            panic!("MetaList must have at least one element");
                                        };

                                        let additions = lits
                                            .map(|x| quote! { self.expect(#x)?; })
                                            .collect::<Vec<_>>();

                                        codes.push(quote! {
                                          let #ident = if #condition {
                                            #(#additions)*
                                            true
                                          } else {
                                            false
                                          };
                                        });
                                    }
                                    Meta::NameValue(_) => {
                                        panic!("NameValue pair is unsupported")
                                    }
                                }
                            }
                            MetaOrLitStr::LitStr(lit) => {
                                codes.push(quote! {
                                  self.expect(#lit)?;
                                });
                            }
                        }
                    }
                }
            }

            for field in &data_struct.fields {
                let field_name = &field.ident;
                fields.push(quote! { #field_name, });
            }

            quote! {
                  #(#codes)*
                  Ok(#struct_name {
                    #(#fields)*
                  })
            }
        }

        syn::Data::Enum(data_enum) => {
            let mut codes = Vec::new();

            let mut variant_names = vec![];
            for varient in &data_enum.variants {
                let varient_name = &varient.ident;
                let mut has_match = false;
                for attr in &varient.attrs {
                    if attr.path().is_ident("matches") {
                        let lit: LitStr = attr
                            .parse_args()
                            .expect("matches must contain a string literal");
                        variant_names.push(quote! {#lit,});
                        codes.push(
                            quote! { #lit => {self.consume(); Ok(#struct_name::#varient_name) } },
                        );
                        has_match = true;
                    }
                }
                if !has_match {
                    panic!("must provide at least one #[matches(...)] for Enum varient");
                }
            }

            quote! {
              let token = self.token()?;
              match &token.to_uppercase()[..] {
                #(#codes)*

                _ => Err(ParseError {
                  reason: format!("Unknown varient: '{}', valid options are {:?}", token.raw, [#(#variant_names)*]),
                  position: Some(token.column),
                })
              }
            }
        }
        _ => panic!("TryParse can only be derived for structs or enums."),
    };

    quote! {
      impl<T> TryParse<#struct_name> for T where T: TokenStream {
        fn try_parse(&mut self) -> ParseResult<#struct_name> {
          #expanded
        }
      }
    }
    .into()
}

#[proc_macro_derive(Params, attributes(param))]
pub fn params_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(data_struct) => &data_struct.fields,
        _ => panic!("Params can only be derived for structs"),
    };

    let mut prefixs = Vec::new();
    let mut field_assignments = Vec::new();
    let mut match_cases = Vec::new();
    let mut after_loop = Vec::new();

    for field in fields {
        let field_name = field.ident.clone();
        let mut param_name = None;

        let is_bool = match &field.ty {
            syn::Type::Path(path) => path.path.is_ident("bool"),
            _ => false,
        };
        let is_option = match &field.ty {
            syn::Type::Path(path) => {
                path.path.leading_colon.is_none()
                    && path.path.segments.len() == 1
                    && path.path.segments[0].ident == "Option"
            }
            _ => false,
        };

        for attr in &field.attrs {
            if attr.path().is_ident("param") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("name") {
                        let lit: LitStr = meta.value()?.parse()?;
                        param_name = Some(lit);
                        return Ok(());
                    }
                    Err(meta.error("unsupported param value"))
                })
                .expect("Unknown param meta");
            }
        }

        let param_name = param_name.expect("must provide #[param(name = ...)] for this feild");

        if is_bool {
            prefixs.push(quote! { let mut #field_name = false; });
            match_cases.push(quote! { #param_name => { self.consume(); #field_name = true; } });
        } else if is_option {
            prefixs.push(quote! { let mut #field_name = None; });
            match_cases.push(quote! {
              #param_name => {
                self.consume();
                self.expect("=")?;
                #field_name = Some(self.try_parse()?);
              }
            });
        } else {
            prefixs.push(quote! { let mut #field_name = None; });
            match_cases.push(quote! {
              #param_name => {
                self.consume();
                self.expect("=")?;
                #field_name = Some(self.try_parse()?);
              }
            });
            after_loop.push(quote! {
              let #field_name = #field_name.ok_or_else(|| ParseError {
                reason: format!("Missing param `{}'", #param_name),
                position: None,
              })?;
            });
        }
        field_assignments.push(quote! { #field_name, });
    }

    let main_loop = quote! {
      while !self.is_eof() {
        let token = self.token()?;
        match &token.to_uppercase()[..] {
          #(#match_cases)*
          _ => break,
        }
      }
    };

    let expanded = quote! {
      impl<T> TryParse<#struct_name> for T where T: TokenStream {
        fn try_parse(&mut self) -> ParseResult<#struct_name> {
          #(#prefixs)*
          #main_loop
          #(#after_loop)*
          Ok(#struct_name {
            #(#field_assignments)*
          })
        }
      }
    };

    expanded.into()
}

#[proc_macro_derive(ExposeNodes)]
pub fn expose_nodes_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(data_struct) => &data_struct.fields,
        _ => panic!("ExposeNodes can only be derived for structs"),
    };

    let mut field_assignments = Vec::new();

    for field in fields {
        let field_name = field.ident.clone();

        match &field.ty {
            syn::Type::Path(path) => {
                if path.path.is_ident("Node") {
                    field_assignments.push(quote! {
                      ret.push(self.#field_name.clone());
                    });
                }
            }
            _ => {}
        }
    }

    let expanded = quote! {
      impl ExposeNodes for #struct_name {
        fn nodes(&self) -> Vec<Node> {
          let mut ret = Vec::new();
          #(#field_assignments)*;
          ret
        }
      }
    };

    expanded.into()
}

#[proc_macro_derive(PartialParse)]
pub fn partial_parse_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let struct_name = format_ident!("{}Partial", struct_name);

    let mut new_struct = vec![];
    new_struct.push(quote! {pub position: u32,});

    let vis = input.vis.to_token_stream();
    let (expanded, return_code, return_with_elements) = match &input.data {
        syn::Data::Struct(data_struct) => {
            let mut codes = Vec::new();
            let mut fields = Vec::new();

            for attr in &input.attrs {
                if attr.path().is_ident("grammar") {
                    let nested = attr
                        .parse_args_with(Punctuated::<MetaOrLitStr, Token![,]>::parse_terminated)
                        .unwrap();

                    for elem in nested {
                        match elem {
                            MetaOrLitStr::Meta(meta) => {
                                match meta {
                                    // #[grammar(node)]
                                    Meta::Path(path) => {
                                        let ident = path.get_ident().unwrap();
                                        codes.push(quote! {
                                          //TODO: `None` for EOF.
                                          //let #ident = match self.try_parse() {
                                          //  Ok(k) => Some(k),
                                          //  Err(e) => {
                                          //    if e.reason.contains("EOF") || is_eof {
                                          //      None
                                          //    } else {
                                          //      Err(e)?
                                          //    }
                                          //  }
                                          //};
                                          let #ident = if !is_eof {
                                            position += 1;
                                            let this = self.position();
                                            match self.try_parse() {
                                              Ok(k) => Some(k),
                                              Err(e) => {
                                                if e.reason.contains("EOF") {
                                                  is_eof = true;
                                                  None
                                                } else {
                                                  Err(e)?
                                                }
                                              }
                                            }
                                          } else {
                                            None
                                          };
                                        });
                                    }
                                    // #[grammar(op("/", "CSDF"))]
                                    Meta::List(list) => {
                                        let ident = list.path.get_ident().unwrap();
                                        let mut lits = list
                                            .parse_args_with(
                                                Punctuated::<LitStr, Token![,]>::parse_terminated,
                                            )
                                            .unwrap()
                                            .into_iter();

                                        let condition = if let Some(first) = lits.next() {
                                            quote! {
                                              self.matches_consume(#first);
                                              self.position += 1;
                                            }
                                        } else {
                                            panic!("MetaList must have at least one element");
                                        };

                                        let additions = lits
                                            .map(|x| quote! { self.expect(#x)?; })
                                            .collect::<Vec<_>>();

                                        codes.push(quote! {
                                          let #ident = if #condition {
                                            #(#additions)*
                                            true
                                          } else {
                                            false
                                          };
                                        });
                                    }
                                    Meta::NameValue(_) => {
                                        panic!("Name-Value pair is unsupported here")
                                    }
                                }
                            }
                            MetaOrLitStr::LitStr(lit) => {
                                codes.push(quote! {
                                  self.expect(#lit)?;
                                });
                            }
                        }
                    }
                }
            }

            for field in &data_struct.fields {
                let field_name = &field.ident;
                let field_vis = &field.vis;
                let field_type = &field.ty;
                fields.push(quote! { #field_name, });
                new_struct.push(quote! {#field_vis #field_name: Option<#field_type>,});
            }

            (
                quote! {
                  let mut position: u32 = 0;
                  #(#codes)*
                },
                quote! {
                  Ok(#struct_name {
                        #(#fields)*
                        position
                  })
                },
                quote! {
                  Ok((#struct_name {
                    #(#fields)*
                    position
                  }, elements))

                },
            )
        }

        syn::Data::Enum(data_enum) => {
            panic!("derive PartialParse is only for enum.");
        }
        _ => panic!("PartialParse can only be derived for structs or enums."),
    };

    quote! {
      #[derive(Clone, Debug, Serialize)]
      #vis struct #struct_name {
        #(#new_struct)*
      }

      impl<T> PartialParse<#struct_name> for T where T: TokenStream {
        fn try_partial(&mut self) -> ParseResult<#struct_name> {
          let mut is_eof = false;
          let elements: Vec<Element> = vec![];
          #expanded
          #return_code
        }

        fn info(&mut self) -> ParseResult<(#struct_name, Vec<Element>)> {
          let mut is_eof = false;
          let elements: Vec<Element> = vec![];
          #expanded;
          #return_with_elements
        }
      }
    }
    .into()
}
