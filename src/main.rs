use json::object;
use regex::Regex;
use syn::{ File, Item, spanned::Spanned, Stmt, Expr, Local };
use proc_macro2::Span;

#[derive(strum_macros::Display, Clone)]
enum PointerKind {
    Inferred,
    NonePointerKind,
    Move,
    Mut,
    Const,
    RawMut,
    RawMov,
    RawConst,
}

static REGEX_LIST: [(&str, PointerKind); 6] = [
    (r"^Option<Box<(.*)>>$", PointerKind::Move),
    (r"^Option<&mut(.*)>$", PointerKind::Mut),
    (r"^Option<&(.*)>$", PointerKind::Const),
    (r"^\*const (.*)$", PointerKind::RawConst),
    (r"^\*mut /\* owning \*/(.*)$", PointerKind::RawMov),
    (r"^\*mut (.*)", PointerKind::RawMut),
];

fn lines_offsets(s: &String) -> Vec<usize> {
    let mut lines = vec![0];
    let mut total = 0;

    for ch in s.chars() {
        total += 1;
        if ch == '\n' {
            lines.push(total);
        }
    }

    lines
}

fn span_to_str(span: &Span, src: &String) -> String {
    let start = span.start();
    let end = span.end();
    let start_line = start.line;
    let end_line = end.line;
    let start_col = start.column;
    let end_col = end.column;

    let lines_offsets = lines_offsets(src);
    let start_offset = lines_offsets[start_line-1] + start_col;
    let end_offset = lines_offsets[end_line-1] + end_col;

    src[start_offset..end_offset].to_string()
}

fn type_to_string(ty: &syn::Type, src: &String) -> String {
    span_to_str(&ty.span(), src)
}

fn parse_type(ty: &syn::Type, src: &String) -> PointerKind {
    let source_text = type_to_string(ty, src);
    for (pattern, kind) in REGEX_LIST.iter() {
        let re = Regex::new(pattern).unwrap();
        if re.is_match(&source_text.as_str()) {
            return kind.clone();
        }
    }
    return PointerKind::NonePointerKind;
}

fn parse_return_type(ret_type: &syn::ReturnType, src: &String) -> PointerKind {
    match ret_type {
        syn::ReturnType::Default => PointerKind::Inferred,
        syn::ReturnType::Type(_, ty) => parse_type(&*ty, src),
    }
}

trait ContainsVariables {
    fn variables(&self) -> Vec<&Local>;
}

impl ContainsVariables for syn::Expr {
    fn variables(&self) -> Vec<&Local> {
        match self {
            Expr::Block(inner) => inner.block.stmts.iter().map(|stmt| stmt.variables()).flatten().collect(),
            Expr::Const(inner) => inner.block.stmts.iter().map(|stmt| stmt.variables()).flatten().collect(),
            Expr::Group(inner) => inner.expr.variables(),
            Expr::Unsafe(inner) => inner.block.stmts.iter().map(|stmt| stmt.variables()).flatten().collect(),
            Expr::ForLoop(inner) => inner.body.stmts.iter().map(|stmt| stmt.variables()).flatten().collect(),
            Expr::Loop(inner) => inner.body.stmts.iter().map(|stmt| stmt.variables()).flatten().collect(),
            Expr::While(inner) => inner.body.stmts.iter().map(|stmt| stmt.variables()).flatten().collect(),
            Expr::If(expr_if) => {
                let mut variables: Vec<&Local> = expr_if.then_branch.stmts.iter().map(|stmt| stmt.variables()).flatten().collect();
                if let Some((_, else_branch)) = &expr_if.else_branch {
                    variables.extend((*else_branch).variables());
                }
                variables
            },
            _ => vec![],
        }
    }
}

impl ContainsVariables for Stmt {
    fn variables(&self) -> Vec<&Local> {
        match self {
            Stmt::Local(local) => vec![local],
            Stmt::Expr(exp, _) => exp.variables(),
            _ => vec![],
        }
    }
}

fn pattern_to_name(pat: &syn::Pat) -> String {
    match pat {
        syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
        syn::Pat::Type(pat_type) => pattern_to_name(&*pat_type.pat),
        _ => panic!("Unsupported pattern"),
    }
}

fn pattern_to_type(pat: &syn::Pat) -> &syn::Type {
    match pat {
        syn::Pat::Type(pat_type) => &*pat_type.ty,
        _ => panic!("Unsupported pattern"),
    }
}

fn parse_function(item_fn: &syn::ItemFn, src: &String) -> json::JsonValue {
    object! {
        kind: "fn",
        name: item_fn.sig.ident.to_string(),
        return_type: parse_return_type(&item_fn.sig.output, src).to_string(),
        args: item_fn.sig.inputs
            .iter()
            .map(|arg| {
                match arg {
                    syn::FnArg::Typed(pat_type) => object! {
                        name: pattern_to_name(&pat_type.pat),
                        "type": parse_type(&pat_type.ty, src).to_string()
                    },
                    _ => panic!("Unsupported argument kind"),
                }
            })
            .collect::<json::Array>(),
        variables: item_fn.block.stmts.iter().map(|stmt| stmt.variables()).flatten().collect::<Vec<&Local>>().iter().map(|local| {
            object! {
                name: pattern_to_name(&local.pat),
                "type": parse_type(pattern_to_type(&local.pat), src).to_string(),
            }
        }).collect::<json::Array>(),
    }
}

fn parse_item(item: &Item, src: &String) -> json::JsonValue {
    match item {
        Item::Fn(item_fn) => parse_function(item_fn, src),
        Item::Struct(item_struct) => object! {
            kind: "struct",
            name: item_struct.ident.to_string(),
            fields: item_struct.fields.iter().map(|field| {
                object! {
                    name: field.ident.as_ref().unwrap().to_string(),
                    "type": parse_type(&field.ty, src).to_string(),
                }
            }).collect::<json::Array>(),
        },
        _ => panic!("Unsupported item kind"),
    }
}

fn parse_file(file: &File, src: &String) -> json::Array {
    (&file.items)
        .into_iter()
        .filter(|item| match item {
            Item::Fn(_) | Item::Struct(_) => true,
            _ => false,
        })
        .map(|item| parse_item(item, src))
        .collect::<json::Array>()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let obj = args
        .iter()
        .skip(1)
        .map(|path| {
            let src = std::fs::read_to_string(path).unwrap();
            let file = syn::parse_file(&src).unwrap();
            object! {
                path: path.as_str(),
                content: parse_file(&file, &src)
            }
        })
        .collect::<json::Array>();

    println!("{}", json::stringify_pretty(obj, 4));
}
