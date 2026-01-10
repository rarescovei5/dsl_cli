use proc_macro2::TokenTree;
use syn::{
    Expr, Ident, LitStr, Token, Type, braced,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

use crate::utils::is_optional_type;

pub struct CliDsl {
    pub(crate) name: LitStr,
    pub(crate) version: LitStr,
    pub(crate) description: LitStr,
    pub(crate) commands: Vec<Command>,
}

pub struct Command {
    pub(crate) name: Ident,
    pub(crate) description: Option<LitStr>,
    pub(crate) arguments: Vec<Argument>,
    pub(crate) options: Vec<CliOption>,
}

pub struct Argument {
    pub(crate) name: Ident,
    pub(crate) description: Option<LitStr>,
    pub(crate) ty: Type,
    pub(crate) default: Option<Expr>,
}

pub struct CliOption {
    pub(crate) flags: LitStr,
    pub(crate) description: Option<LitStr>,
    pub(crate) arguments: Vec<Argument>,
    pub(crate) required: bool,
}

// ----------------------------------------------------------------
// Parsing Implementation
// ----------------------------------------------------------------

impl Parse for CliDsl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse: name "value",
        let _: Ident = input.parse()?; // "name"
        let name: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        // Parse: version "value",
        let _: Ident = input.parse()?; // "version"
        let version: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        // Parse: description "value",
        let _: Ident = input.parse()?; // "description"
        let description: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        // Parse commands
        let mut commands = Vec::new();
        while !input.is_empty() {
            commands.push(input.parse()?);
        }

        Ok(CliDsl {
            name,
            version,
            description,
            commands,
        })
    }
}

impl Parse for Command {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse: cmd <name> ["description"] { ... }
        let cmd_keyword: Ident = input.parse()?;
        if cmd_keyword != "cmd" {
            return Err(syn::Error::new(cmd_keyword.span(), "expected 'cmd'"));
        }

        let name: Ident = input.parse()?;

        // Optional description
        let description = if input.peek(LitStr) {
            Some(input.parse()?)
        } else {
            None
        };

        // Parse body
        let content;
        braced!(content in input);

        let mut arguments = Vec::new();
        let mut options = Vec::new();

        while !content.is_empty() {
            let keyword: Ident = content.parse()?;
            match keyword.to_string().as_str() {
                "arg" => {
                    arguments.push(parse_argument(&content, true, true)?);
                }
                "opt" => {
                    options.push(parse_option(&content, false)?);
                }
                "req_opt" => {
                    options.push(parse_option(&content, true)?);
                }
                _ => {
                    return Err(syn::Error::new(
                        keyword.span(),
                        format!("unexpected keyword '{}'", keyword),
                    ));
                }
            }
        }

        // Optional trailing comma after the command block
        let _ = input.parse::<Token![,]>();

        Ok(Command {
            name,
            description,
            arguments,
            options,
        })
    }
}

fn parse_argument(
    input: ParseStream,
    is_positional: bool,
    is_ctx_required: bool,
) -> syn::Result<Argument> {
    // arg <name> ["description"] [: type] [= <default>],
    let name: Ident = input.parse()?;

    // Optional description
    let description = if input.peek(LitStr) {
        if !is_positional {
            return Err(syn::Error::new(
                input.span(),
                "For arguments in options, a description is not required. Consider adding info about arguments in the option description directly.",
            ));
        }
        Some(input.parse()?)
    } else {
        None
    };

    // Parse type (default is String)
    // Parse type (default is String)
    let ty: Type = if input.is_empty() || input.peek(Token![,]) || input.peek(Token![=]) {
        syn::parse_quote!(String)
    } else if input.peek(Token![:]) {
        input.parse::<Token![:]>()?;
        input.parse()?
    } else {
        let unexpected: TokenTree = input.parse()?;
        return Err(syn::Error::new(
            unexpected.span(),
            format!("expected ':', got {}", unexpected),
        ));
    };
    let ty_is_option = is_optional_type(&ty);

    // Optional default value
    let default = if input.peek(Token![=]) {
        let asignment = input.parse::<Token![=]>()?;

        if !ty_is_option && is_ctx_required {
            return Err(syn::Error::new(
                asignment.span(),
                "Default values are only allowed for *Option<T>* types or for arguments *inside optional options*.",
            ));
        }

        Some(input.parse()?)
    } else {
        None
    };

    // Optional trailing comma
    let _ = input.parse::<Token![,]>();

    Ok(Argument {
        name,
        description,
        ty,
        default,
    })
}

fn parse_option(input: ParseStream, required: bool) -> syn::Result<CliOption> {
    // opt|req_opt "<flags>" ["description"] [{args}],
    let flags: LitStr = input.parse()?;

    // Optional description
    let description = if input.peek(LitStr) {
        Some(input.parse()?)
    } else {
        None
    };

    // Optional arguments block
    let arguments = if input.peek(syn::token::Brace) {
        let content;
        braced!(content in input);
        let mut args = Vec::new();
        while !content.is_empty() {
            let keyword: Ident = content.parse()?;
            if keyword == "arg" {
                args.push(parse_argument(&content, false, required)?);
            } else {
                return Err(syn::Error::new(
                    keyword.span(),
                    "expected 'arg' inside option block",
                ));
            }
        }
        args
    } else {
        Vec::new()
    };

    // Optional trailing comma
    let _ = input.parse::<Token![,]>();

    Ok(CliOption {
        flags,
        description,
        arguments,
        required,
    })
}
