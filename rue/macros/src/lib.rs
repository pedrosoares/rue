use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{braced, Expr, Ident, LitStr, Token};

/// Parsed HTML template
struct HtmlTemplate {
    nodes: Vec<HtmlNode>,
}

enum HtmlNode {
    Element {
        tag: String,
        attrs: Vec<HtmlAttr>,
        children: Vec<HtmlNode>,
        self_closing: bool,
    },
    Text(String),
    Fragment(Vec<HtmlNode>),
    Expr(Expr),
    /// A VNode expression — `{vnode expr}` evaluates to a VNode directly
    VNodeExpr(Expr),
}

struct HtmlAttr {
    name: String,
    value: HtmlAttrValue,
}

enum HtmlAttrValue {
    Static(String),
    Dynamic(Expr),
    Event(Expr),
    Bool,
}

impl Parse for HtmlTemplate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let nodes = parse_nodes(input, false)?;
        Ok(HtmlTemplate { nodes })
    }
}

/// Parse a sequence of HTML nodes until a closing tag or end of input.
fn parse_nodes(input: ParseStream, inside_tag: bool) -> syn::Result<Vec<HtmlNode>> {
    let mut nodes = Vec::new();

    while !input.is_empty() {
        if inside_tag {
            if input.peek(Token![<]) && input.peek2(Token![/]) {
                break;
            }
        }

        if input.peek(Token![<]) {
            let fork = input.fork();
            let _ = fork.parse::<Token![<]>();
            if fork.peek(Token![>]) {
                // Fragment <> ... </>
                let _ = input.parse::<Token![<]>();
                let _ = input.parse::<Token![>]>();
                let children = parse_nodes(input, false)?;
                let _ = input.parse::<Token![<]>();
                let _ = input.parse::<Token![/]>();
                let _ = input.parse::<Token![>]>();
                nodes.push(HtmlNode::Fragment(children));
                continue;
            }

            nodes.push(parse_element(input)?);
        } else if input.peek(syn::token::Brace) {
            let content;
            let _ = braced!(content in input);

            // Check for `vnode` prefix: {vnode expr} or {vnode: expr}
            // This tells the macro the expression returns a VNode directly
            if content.peek(Ident) && content.fork().parse::<Ident>().ok().map_or(false, |id| id == "vnode") {
                // Consume the "vnode" keyword
                let _: Ident = content.parse()?;
                // Optionally consume `:`
                if content.peek(Token![:]) {
                    let _: Token![:] = content.parse()?;
                }
                let expr: Expr = content.parse()?;
                nodes.push(HtmlNode::VNodeExpr(expr));
            } else {
                // Regular expression — treat as text
                let expr: Expr = content.parse()?;
                nodes.push(HtmlNode::Expr(expr));
            }
        } else {
            // Text content
            let mut text = String::new();
            while !input.is_empty() && !input.peek(Token![<]) && !input.peek(syn::token::Brace) {
                if let Ok(lit) = input.parse::<LitStr>() {
                    text.push_str(&lit.value());
                } else {
                    let fork = input.fork();
                    if let Ok(ident) = fork.parse::<Ident>() {
                        text.push_str(&ident.to_string());
                        text.push(' ');
                        input.parse::<Ident>().ok();
                    } else if let Ok(remaining) = input.fork().parse::<proc_macro2::TokenStream>()
                    {
                        let s = remaining.to_string();
                        if !s.is_empty() {
                            text.push_str(&s);
                            input.parse::<proc_macro2::TokenStream>().ok();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            if !text.is_empty() {
                let text = text.trim_end().to_string();
                nodes.push(HtmlNode::Text(text));
            } else {
                break;
            }
        }
    }

    Ok(nodes)
}

fn parse_element(input: ParseStream) -> syn::Result<HtmlNode> {
    let _ = input.parse::<Token![<]>();
    let tag: Ident = input.parse()?;
    let tag_str = tag.to_string();

    let mut attrs = Vec::new();
    let mut self_closing = false;

    // Parse attributes
    while !input.peek(Token![>]) && !input.peek(Token![/]) {
        if input.peek(syn::token::Brace) {
            let content;
            let _ = braced!(content in input);
            let _: Expr = content.parse()?;
            continue;
        }

        // Parse attribute name — may contain hyphens (e.g. "stroke-linecap")
        let first_ident: Ident = input.parse()?;
        let mut name_str = first_ident.to_string();
        // Consume any `-ident` suffixes (for hyphenated HTML attributes)
        while input.peek(Token![-]) {
            let _ = input.parse::<Token![-]>();
            let part: Ident = input.parse()?;
            name_str.push('-');
            name_str.push_str(&part.to_string());
        }

        // Event handler: on:click={...}
        if name_str == "on" && input.peek(Token![:]) {
            let _ = input.parse::<Token![:]>();
            let event_ident: Ident = input.parse()?;
            let full_name = format!("on:{}", event_ident);

            if input.peek(Token![=]) {
                let _ = input.parse::<Token![=]>();
            }
            if input.peek(syn::token::Brace) {
                let content;
                let _ = braced!(content in input);
                let handler: Expr = content.parse()?;
                attrs.push(HtmlAttr {
                    name: full_name,
                    value: HtmlAttrValue::Event(handler),
                });
            }
            continue;
        }

        if input.peek(Token![=]) {
            let _ = input.parse::<Token![=]>();

            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                attrs.push(HtmlAttr {
                    name: name_str,
                    value: HtmlAttrValue::Static(lit.value()),
                });
            } else if input.peek(syn::token::Brace) {
                let content;
                let _ = braced!(content in input);
                let expr: Expr = content.parse()?;
                attrs.push(HtmlAttr {
                    name: name_str,
                    value: HtmlAttrValue::Dynamic(expr),
                });
            }
        } else {
            attrs.push(HtmlAttr {
                name: name_str,
                value: HtmlAttrValue::Bool,
            });
        }
    }

    if input.peek(Token![/]) {
        let _ = input.parse::<Token![/]>();
        self_closing = true;
    }

    let _ = input.parse::<Token![>]>();

    let children = if self_closing {
        vec![]
    } else {
        let children = parse_nodes(input, true)?;
        let _ = input.parse::<Token![<]>();
        let _ = input.parse::<Token![/]>();
        let _: Ident = input.parse()?;
        let _ = input.parse::<Token![>]>();
        children
    };

    Ok(HtmlNode::Element {
        tag: tag_str,
        attrs,
        children,
        self_closing,
    })
}

impl ToTokens for HtmlTemplate {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let node_tokens = self.nodes.iter().map(|node| node.as_statement());
        let expanded = quote! {
            {
                let mut __rue_children: ::std::vec::Vec<rue_core::node::VNode> = ::std::vec::Vec::new();
                #(#node_tokens)*
                if __rue_children.len() == 1 {
                    __rue_children.into_iter().next().unwrap()
                } else {
                    rue_core::node::VNode::fragment(__rue_children)
                }
            }
        };
        tokens.extend(expanded);
    }
}

impl HtmlNode {
    /// Generate a statement that pushes this node to __rue_children.
    fn as_statement(&self) -> proc_macro2::TokenStream {
        match self {
            HtmlNode::Text(text) => {
                let text = text.clone();
                quote! {
                    __rue_children.push(rue_core::node::VNode::text(#text));
                }
            }
            HtmlNode::Expr(expr) => {
                quote! {
                    __rue_children.push({
                        let __val = (#expr);
                        rue_core::node::VNode::text(&__val.to_string())
                    });
                }
            }
            HtmlNode::VNodeExpr(expr) => {
                // Push the VNode expression directly — caller must return VNode
                quote! {
                    __rue_children.push({
                        let __vnode: rue_core::node::VNode = (#expr);
                        __vnode
                    });
                }
            }
            HtmlNode::Fragment(children) => {
                let child_stmts: Vec<_> = children.iter().map(|c| c.as_statement()).collect();
                quote! {
                    {
                        let mut __frag: ::std::vec::Vec<rue_core::node::VNode> = ::std::vec::Vec::new();
                        #(#child_stmts)*
                        __rue_children.push(rue_core::node::VNode::fragment(__frag));
                    }
                }
            }
            HtmlNode::Element { .. } => {
                let expr = self.as_expression();
                quote! {
                    __rue_children.push(#expr);
                }
            }
        }
    }

    /// Generate an expression that evaluates to a VNode.
    fn as_expression(&self) -> proc_macro2::TokenStream {
        match self {
            HtmlNode::Element { tag, attrs, children, self_closing } => {
                let tag_str = tag.as_str();
                let mut builder = quote! {
                    rue_core::node::VNode::element(#tag_str)
                };

                for attr in attrs {
                    match &attr.value {
                        HtmlAttrValue::Static(val) => {
                            let name = attr.name.as_str();
                            let val = val.clone();
                            builder = quote! {
                                #builder.attr(#name, #val)
                            };
                        }
                        HtmlAttrValue::Dynamic(expr) => {
                            let name = attr.name.as_str();
                            builder = quote! {
                                #builder.attr(#name, &(#expr).to_string())
                            };
                        }
                        HtmlAttrValue::Event(expr) => {
                            let event_type = &attr.name;
                            let event_type = event_type.strip_prefix("on:").unwrap_or(event_type);
                            builder = quote! {
                                #builder.on(#event_type, #expr)
                            };
                        }
                        HtmlAttrValue::Bool => {
                            let name = attr.name.as_str();
                            builder = quote! {
                                #builder.attr(#name, "")
                            };
                        }
                    }
                }

                if !children.is_empty() && !*self_closing {
                    let child_exprs: Vec<_> = children.iter().map(|c| c.as_expression()).collect();
                    builder = quote! {
                        #builder.children(::std::vec![#(#child_exprs),*])
                    };
                }

                quote! { #builder.build() }
            }
            HtmlNode::Text(text) => {
                let text = text.clone();
                quote! { rue_core::node::VNode::text(#text) }
            }
            HtmlNode::Expr(expr) => {
                quote! {{
                    let __val = (#expr);
                    rue_core::node::VNode::text(&__val.to_string())
                }}
            }
            HtmlNode::VNodeExpr(expr) => {
                quote! {{
                    let __vnode: rue_core::node::VNode = (#expr);
                    __vnode
                }}
            }
            HtmlNode::Fragment(children) => {
                let child_exprs: Vec<_> = children.iter().map(|c| c.as_expression()).collect();
                quote! { rue_core::node::VNode::fragment(::std::vec![#(#child_exprs),*]) }
            }
        }
    }
}

/// Entry point: html! { ... } macro
#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    let template = syn::parse_macro_input!(input as HtmlTemplate);
    let expanded = template.to_token_stream();
    TokenStream::from(expanded)
}

/// Component attribute macro (for future use)
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
