#![feature(plugin_registrar, rustc_private, quote)]

extern crate rustc;
extern crate rustc_errors;
extern crate rustc_plugin;
extern crate syntax;

use std::collections::VecDeque;

use rustc_plugin::Registry;

use syntax::ptr::P;
use syntax::ast::{Expr, ExprKind, Pat, Stmt, StmtKind, Ident, Path};
// use syntax::tokenstream::TokenTree;
use syntax::tokenstream::{TokenTree, TokenStream, ThinTokenStream};
use syntax::parse::PResult;
use syntax::parse::token::Token;
use syntax::parse::parser::Parser;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::{DummyResult, ExtCtxt, MacResult, MacEager};

use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::ToTokens;

use syntax::symbol::Symbol;
use syntax::ext::base::{SyntaxExtension, Annotatable};
use syntax::ast::{ItemKind, MetaItem, FnDecl, PatKind, SpannedIdent};
use syntax::codemap::Spanned;

macro_rules! debug {
    ($($t:tt)*) => (
        if ::std::env::var_os("PEAR_CODEGEN_DEBUG").is_some() {
            println!($($t)*);
        }
    )
}

/// Compiler hook for Rust to register plugins.
#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("parse", parse_macro_outer);
    reg.register_syntax_extension(Symbol::intern("parser"),
        SyntaxExtension::MultiModifier(Box::new(parser_decorator)));
}

fn get_input_from_decl(ecx: &ExtCtxt, decl: &FnDecl) -> SpannedIdent {
    let pat = &decl.inputs[0].pat;
    match pat.node {
        PatKind::Ident(_, ident, _) => return ident,
        _ => ecx.span_err(pat.span, "expected an identifier")
    }

    Spanned { node: Ident::from_str("__dummy"), span: pat.span }
}

fn parser_decorator(ecx: &mut ExtCtxt,
                    sp: Span,
                    attr: &MetaItem,
                    annotated: Annotatable
                   ) -> Annotatable {
    if attr.is_value_str() || attr.is_meta_item_list() {
        ecx.span_err(sp, "the `parser` attribute does not support any parameters");
    }

    if let Annotatable::Item(ref item) = annotated {
        if let ItemKind::Fn(ref decl, safety, cness, abi, ref generics, ref block) = item.node {
            let input = get_input_from_decl(ecx, decl);
            let new_inner_fn = quote_expr!(ecx, parse!($input, $block));
            let new_block = ecx.block_expr(new_inner_fn);
            let node = ItemKind::Fn(decl.clone(), safety, cness, abi, generics.clone(), new_block);

            let mut new_item = item.clone().into_inner();
            new_item.node = node;

            if block.stmts.len() > 6 {
                new_item.attrs.push(quote_attr!(ecx, #[inline]));
            } else {
                new_item.attrs.push(quote_attr!(ecx, #[inline(always)]));
            }

            return Annotatable::Item(P(new_item))
        }
    }

    let item_span = match annotated {
        Annotatable::Item(ref item) => item.span,
        Annotatable::TraitItem(ref item) => item.span,
        Annotatable::ImplItem(ref item) => item.span
    };

    ecx.struct_span_err(sp, "this attribute can only be applied to functions")
        .span_note(item_span, "the attribute was applied to this item")
        .emit();

    annotated
}

fn parse_macro_outer(ecx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
    let parser = ecx.new_parser_from_tts(args);
    let expr = match parse_macro(parser, ecx, sp) {
        Ok(expr) => expr,
        Err(mut diag) => {
            diag.emit();
            return DummyResult::expr(sp);
        }
    };

    debug!("Returning: {:?}", expr);
    MacEager::expr(expr)
}

fn parse_macro<'a>(mut parser: Parser<'a>, ecx: &mut ExtCtxt<'a>, _: Span) -> PResult<'a, P<Expr>> {
    let input_expr = parser.parse_expr()?;
    parser.expect(&Token::Comma)?;
    let output_expr = parser.parse_expr()?;
    parser.expect(&Token::Eof)?;

    let wild = ecx.pat_wild(DUMMY_SP);
    Ok(gen_expr(ecx, &input_expr, &wild, &output_expr, VecDeque::new()))
}

static FN_PENULTIMATE_WHITELIST: &'static [&'static str] = &["str"];
static FN_END_WHITELIST: &'static [&'static str] = &["drop", "from_utf8"];

static MACRO_WHITELIST: &'static [&'static str] = &[
    "println", "format", "panic", "print",  "vec", "write", "writeln",
    "unimplemented", "unreachable", "assert", "assert_eq", "assert_ne",
    "debug_assert", "debug_assert_eq", "debug_assert_ne"
];

fn is_whitelisted_fn(expr: &P<Expr>) -> bool {
    if let ExprKind::Path(_, ref path) = expr.node {
        // Check the penultimate segment, if there is one.
        let num_segs = path.segments.len();
        if num_segs > 1 {
            let penultimate = path.segments[num_segs - 2].identifier.name.as_str();
            let is_whitelisted = penultimate.starts_with(char::is_uppercase)
                || FN_PENULTIMATE_WHITELIST.iter().any(|v| &&*penultimate == v);
            if is_whitelisted { return true; }
        }

        // Check the last segment.
        let last = path.segments[num_segs - 1].identifier.name.as_str();
        last.starts_with(char::is_uppercase)
            || FN_END_WHITELIST.iter().any(|v| &&*last == v)
    } else {
        false
    }
}

fn is_whitelisted_macro(path: &Path) -> bool {
    let first_ident = path.segments[0].identifier.name.as_str();
    MACRO_WHITELIST.iter().any(|val| &&*first_ident == val)
}

fn get_ident(num: usize) -> Ident {
    let chars = "abcdefghijklmnopqrstuvwxyz";
    let available = (chars.len() * (chars.len() + 1)) / 2;
    if num >= available {
        panic!("An expression contained more than {} subexpressions! \
               Please report this error.", available)
    }

    let (mut need, mut start) = (1, num);
    while (start + need) > chars.len() {
        start -= 26 - (need - 1);
        need += 1;
    }

    Ident::from_str(&chars[start..(start + need)])
}

/// Takes an expression `param`, generates a binding to a unique identifier for
/// every subexpression in `param`, replaces each subexpression in `param` with
/// the new unique identifier, generates a unique identifer for this new,
/// overarching expression, adds every new binding for each new identifier to
/// `stmts`, and returns the rebound expression.
fn remonad_param(ecx: &ExtCtxt, param: P<Expr>, stmts: &mut Vec<Stmt>) -> P<Expr> {
    let mut param_expr = param.clone().into_inner();
    match param_expr.node {
        ExprKind::Call(..) | ExprKind::MethodCall(..) | ExprKind::Mac(..) => {
            let unique_ident = get_ident(stmts.len()); // FIXME: Generate this.
            stmts.push(quote_stmt!(ecx, let $unique_ident = $param;).unwrap());
            ecx.expr_ident(param.span, unique_ident)
        }
        ExprKind::Binary(op, left_expr, right_expr) => {
            let new_left_expr = remonad_param(ecx, left_expr, stmts);
            let new_right_expr = remonad_param(ecx, right_expr, stmts);
            param_expr.node = ExprKind::Binary(op, new_left_expr, new_right_expr);
            P(param_expr)
        }
        ExprKind::Tup(exprs) => {
            let mut new_exprs = Vec::new();
            for expr in exprs {
                new_exprs.push(remonad_param(ecx, expr, stmts));
            }

            param_expr.node = ExprKind::Tup(new_exprs);
            P(param_expr)
        }
        ExprKind::AddrOf(mutability, expr) => {
            let new_expr = remonad_param(ecx, expr, stmts);
            param_expr.node = ExprKind::AddrOf(mutability, new_expr);
            P(param_expr)
        }
        ExprKind::Cast(expr, ty) => {
            let new_expr = remonad_param(ecx, expr, stmts);
            param_expr.node = ExprKind::Cast(new_expr, ty);
            P(param_expr)
        }
        ExprKind::Index(expr, index) => {
            let new_expr = remonad_param(ecx, expr, stmts);
            param_expr.node = ExprKind::Index(new_expr, index);
            P(param_expr)
        }
        ExprKind::Path(..) | ExprKind::Lit(..) | ExprKind::Closure(..) => {
            param
        }
        _ => {
            debug!("not lifting: {:?}", param.node);
            ecx.span_warn(param.span, "remonad: this expression is not being lifted");
            param
        }
    }
}

/// Monadifies the set of expressions in `params`.
///
///     A set of expressions: [A, B, C]
///
///     Is converted into: { let a = A'; let b = B'; let c = C'; }
///
/// Each prime is the expression run though `remonad_param`.
///
/// The vector of expressions [a, b, c] is passed to `remake` to yield a new
/// expression, `new_expr`, which is inserted at the end of the converted block:
///
///     { let a = A; let b = B; let c = C; new_expr }
///
/// This block is passed to `gen_expr` for monadification of each subexpression.
/// This means that the type of each subexpression remains the same (due to
/// remonadification) but the type of the returned expression from this function
/// is `ParseResult`.
///
/// If the type of `expr` is already a `ParseResult`, `expr_is_end_type` should
/// be set to `true` to avoid re-monadifying the resulting expression.
fn remonad_params<F>(
    ecx: &ExtCtxt,
    input: &P<Expr>,
    binding: &P<Pat>,
    expr: &P<Expr>,
    params: Vec<P<Expr>>,
    expr_is_end_type: bool,
    remake: F,
) -> P<Expr>
    where F: FnOnce(Vec<P<Expr>>) -> ExprKind
{
    debug!("remonadding: {} param", params.len());
    let mut stmts = vec![];
    let new_params: Vec<_> = params.into_iter()
        .map(|p| remonad_param(ecx, p, &mut stmts))
        .collect();

    let mut new_expr = expr.clone().into_inner();
    new_expr.node = remake(new_params);

    // `remonad_params` is co-recursive with `gen_expr` and will be called with
    // `expr` once again, except `expr` will already be monadified. In that
    // case, `stmts` will be empty. This is the base case.
    if stmts.is_empty() {
        let expr = P(new_expr);
        match expr_is_end_type {
            true => expr,
            false => quote_expr!(ecx, ::pear::ParseResult::Done($expr))
        }
    } else {
        debug!("new expr: {:?}", new_expr);
        debug!("statements: {:?}", stmts);
        stmts.push(ecx.stmt_expr(P(new_expr)));
        let block = ecx.expr_block(ecx.block(expr.span, stmts));
        gen_expr(ecx, input, binding, &block, VecDeque::new())
    }
}

/// Entry point: this gets called with the user's expression `expr` and parse
/// input expression `input`. Given any `expr`, returns an expression of type
/// `ParseResult`. If `stmts` is non-empty, an expression or statement is
/// generated for each statement in `stmts` and the expression generated for
/// `expr` is bound to `binding`.
fn gen_expr(
    ecx: &ExtCtxt,
    input: &P<Expr>,
    binding: &P<Pat>,
    expr: &P<Expr>,
    stmts: VecDeque<Stmt>
) -> P<Expr> {
    let mut unwrapped_expr = expr.clone().into_inner();
    let new_expr = match unwrapped_expr.node {
        ExprKind::Call(fn_name, params) => {
            let whitelisted = is_whitelisted_fn(&fn_name);
            if whitelisted {
                debug!("in a whitelisted call");
                let remake = |new_params| ExprKind::Call(fn_name, new_params);
                remonad_params(ecx, input, binding, expr, params, false, remake)
            } else {
                debug!("not whitelisted! inserted input for: {:?}", fn_name);
                let remake = |mut new_params: Vec<P<Expr>>| {
                    // Ensure we don't insert the input twice.
                    if new_params.is_empty() || &new_params[0] != input {
                        new_params.insert(0, input.clone());
                    }

                    ExprKind::Call(fn_name, new_params)
                };

                remonad_params(ecx, input, binding, expr, params, true, remake)
            }
        }
        ExprKind::MethodCall(ty, params) => {
            let remake = |new_params| ExprKind::MethodCall(ty, new_params);
            remonad_params(ecx, input, binding, expr, params, false, remake)
        }
        ExprKind::Block(block) => {
            let stmt = gen_stmt(ecx, input, VecDeque::from(block.stmts.clone()));
            quote_expr!(ecx, { $stmt })
        }
        ExprKind::Mac(mut mac) => {
            if is_whitelisted_macro(&mac.node.path) {
                quote_expr!(ecx, ::pear::ParseResult::Done($expr))
            } else {
                let mut streams: Vec<_> = quote_tokens!(ecx, $input,).into_iter()
                    .map(|tt| TokenStream::from(tt))
                    .collect();

                streams.push(mac.node.stream());
                mac.node.tts = ThinTokenStream::from(TokenStream::concat(streams));
                unwrapped_expr.node = ExprKind::Mac(mac);
                P(unwrapped_expr)
            }
        }
        ExprKind::Tup(exprs) => {
            let remake = |new_exprs| ExprKind::Tup(new_exprs);
            remonad_params(ecx, input, binding, expr, exprs, false, remake)
        }
        ExprKind::TupField(indexed_expr, i) => {
            let remake = |new_expr: Vec<P<Expr>>| ExprKind::TupField(new_expr[0].clone(), i);
            remonad_params(ecx, input, binding, expr, vec![indexed_expr], false, remake)
        }
        ExprKind::Unary(op, uexpr) => {
            let remake = |new_expr: Vec<P<Expr>>| ExprKind::Unary(op, new_expr[0].clone());
            remonad_params(ecx, input, binding, expr, vec![uexpr], false, remake)
        }
        ExprKind::Struct(path, fields, base) => {
            if let Some(ref base) = base {
                ecx.span_warn(base.span, "this expression is not being lifted");
            }

            let exprs: Vec<P<Expr>> = fields.iter()
                .map(|field| field.expr.clone())
                .collect();

            remonad_params(ecx, input, binding, expr, exprs, false, |new_exprs| {
                let new_fields = fields.into_iter()
                    .enumerate()
                    .map(|(i, mut field)| {
                        field.expr = new_exprs[i].clone();
                        field
                    }).collect();

                ExprKind::Struct(path, new_fields, base)
            })
        }
        ExprKind::Break(sp_ident, expr) => {
            if expr.is_some() {
                ecx.span_fatal(unwrapped_expr.span, "unsupported expression");
            }

            unwrapped_expr.node = ExprKind::Break(sp_ident, expr);
            P(unwrapped_expr)
        }
        ExprKind::Ret(expr) => {
            match expr {
                None => ecx.span_fatal(unwrapped_expr.span, "return requires expression"),
                Some(expr) => {
                    let wild = ecx.pat_wild(DUMMY_SP);
                    let new_expr = gen_expr(ecx, input, &wild, &expr, VecDeque::new());
                    unwrapped_expr.node = ExprKind::Ret(Some(new_expr));
                }
            }

            P(unwrapped_expr)
        }
        ExprKind::Continue(..) => {
            P(unwrapped_expr)
        }
        ExprKind::If(cond_expr, block, else_block) => {
            ecx.span_warn(cond_expr.span, "this expression is not being lifted");

            let wild = ecx.pat_wild(DUMMY_SP);
            let new_else = match else_block {
                Some(ref block) => gen_expr(ecx, input, &wild, block, VecDeque::new()),
                None => gen_expr(ecx, input, &wild, &quote_expr!(ecx, ()), VecDeque::new())
            };

            let new_block = gen_stmt(ecx, input, VecDeque::from(block.into_inner().stmts));
            quote_expr!(ecx, if $cond_expr { $new_block } else { $new_else })
        }
        ExprKind::IfLet(pat, pat_expr, true_block, else_block) => {
            ecx.span_warn(pat_expr.span, "this expression is not being lifted");

            let wild = ecx.pat_wild(DUMMY_SP);
            let new_else = match else_block {
                Some(ref block) => gen_expr(ecx, input, &wild, block, VecDeque::new()),
                None => gen_expr(ecx, input, &wild, &quote_expr!(ecx, ()), VecDeque::new())
            };

            let new_block = gen_stmt(ecx, input, VecDeque::from(true_block.into_inner().stmts));
            quote_expr!(ecx, if let $pat = $pat_expr { $new_block } else { $new_else })
        }
        ExprKind::Match(expr, mut arms) => {
            ecx.span_warn(expr.span, "this expression is not being lifted");

            let wild = ecx.pat_wild(DUMMY_SP);
            for arm in &mut arms {
                arm.body = gen_expr(ecx, input, &wild, &arm.body, VecDeque::new());
            }

            unwrapped_expr.node = ExprKind::Match(expr, arms);
            P(unwrapped_expr)
        }
        ExprKind::Assign(left_expr, right_expr) => {
            ecx.span_warn(left_expr.span, "this expression is not being lifted");

            let remake = |new_expr: Vec<P<Expr>>| ExprKind::Assign(left_expr, new_expr[0].clone());
            remonad_params(ecx, input, binding, expr, vec![right_expr], false, remake)
        }
        ExprKind::Cast(cexpr, ty) => {
            let remake = |new_expr: Vec<P<Expr>>| ExprKind::Cast(new_expr[0].clone(), ty);
            remonad_params(ecx, input, binding, expr, vec![cexpr], false, remake)
        }
        ExprKind::Path(..) | ExprKind::Lit(..) => {
            quote_expr!(ecx, ::pear::ParseResult::Done($expr))
        }
        _ => {
            debug!("Not lifting: {:?}", expr.node);
            ecx.span_warn(expr.span, "this expression is being lifted blindly");
            quote_expr!(ecx, ::pear::ParseResult::Done($expr))
        }
    };

    if stmts.is_empty() {
        new_expr
    } else {
        let rest = gen_stmt(ecx, input, stmts);
        quote_expr!(ecx,
            match $new_expr {
                ::pear::ParseResult::Done($binding) => {
                    $rest
                }
                ::pear::ParseResult::Error(e) => ::pear::ParseResult::Error(e)
            }
        )
    }
}

/// Generates an expression or statement for all of the statement in `stmt`. If
/// `stmts` is empty, returns the generated expression (from `gen_expr`) for
/// `()`. This function is co-recursive with `gen_expr`.
fn gen_stmt(ecx: &ExtCtxt, input: &P<Expr>, mut stmts: VecDeque<Stmt>) -> Vec<TokenTree> {
    let wild = ecx.pat_wild(DUMMY_SP);
    let mut stmt = match stmts.pop_front() {
        Some(stmt) => stmt,
        None => {
            debug!("Hitting degenerate case.");
            let expr = gen_expr(ecx, input, &wild, &quote_expr!(ecx, ()), stmts);
            return expr.to_tokens(ecx);
        }
    };

    match stmt.node {
        StmtKind::Local(local) => {
            if local.init.is_some() {
                let expr = local.init.as_ref().unwrap();
                gen_expr(ecx, input, &local.pat, expr, stmts).to_tokens(ecx)
            } else {
                stmt.node = StmtKind::Local(local);
                stmt.to_tokens(ecx)
            }
        }
        StmtKind::Expr(ref expr) => {
            debug!("Parsing regular expr: {:?}", expr);
            gen_expr(ecx, input, &wild, expr, stmts).to_tokens(ecx)
        }
        StmtKind::Semi(ref expr) => {
            // Ensure the type of this is (monadically) a `()`.
            if stmts.is_empty() {
                stmts.push_front(ecx.stmt_expr(quote_expr!(ecx, ())));
            }

            gen_expr(ecx, input, &wild, expr, stmts).to_tokens(ecx)
        }
        StmtKind::Mac(mac_stmt) => {
            let mac = mac_stmt.into_inner().0;
            let mac_expr = P(Expr {
                id: stmt.id,
                node: ExprKind::Mac(mac),
                span: stmt.span,
                attrs: Vec::new().into()
            });

            gen_expr(ecx, input, &wild, &mac_expr, stmts).to_tokens(ecx)
        }
        StmtKind::Item(item) => item.to_tokens(ecx)
    }
}
