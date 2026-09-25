use crate::clippy_utils::res::MaybeDef;
use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::{fn_def_id_with_node_args, sym};
use rustc_ast::LitKind;
use rustc_hir::def::Res;
use rustc_hir::{BorrowKind, Expr, ExprKind, Mutability};
use rustc_lint::{Applicability, LateContext, LateLintPass, declare_lint_pass};
use rustc_middle::ty::Ty;
use rustc_span::Symbol;

declare_clippy_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Example
    /// ```no_run
    /// // example code where clippy issues a warning
    /// ```
    /// Use instead:
    /// ```no_run
    /// // example code which does not raise clippy warning
    /// ```
    #[clippy::version = "1.100.0"]
    pub REF_STRING_FROM_INSTEAD_OF_STR,
    pedantic,
    "default lint description"
}

declare_lint_pass!(RefTringFromInsteadOfStr => [REF_STRING_FROM_INSTEAD_OF_STR]);

// cargo uibless para actualizar los archivos de error (stderr)
impl LateLintPass<'_> for RefTringFromInsteadOfStr {
    fn check_expr(&mut self, cx: &LateContext<'_>, expr: &Expr<'_>) {
        let typeck = cx.typeck_results();
        let str_ref = Ty::new_imm_ref(cx.tcx, cx.tcx.lifetimes.re_erased, cx.tcx.types.str_);

        let (args, param_types): (&[Expr<'_>], &[Ty<'_>]) = match expr.kind {
            ExprKind::MethodCall(_, _, args, _) if let Some(def_id) = typeck.type_dependent_def_id(expr.hir_id) => (
                args,
                &cx.tcx
                    .fn_sig(def_id)
                    //.instantiate(cx.tcx, typeck.node_args(expr.hir_id))
                    .skip_binder()
                    .skip_binder()
                    .inputs()[1..],
            ),
            ExprKind::Call(func, args)
                if let ExprKind::Path(qpath) = func.kind
                    && let Res::Def(_, def_id) = typeck.qpath_res(&qpath, func.hir_id) =>
            {
                (args, cx.tcx.fn_sig(def_id).skip_binder().skip_binder().inputs())
            },
            _ => (&[], &[]),
        };

        // TODO: Ver esto
        let body_owner = cx.tcx.hir_enclosing_body_owner(expr.hir_id);
        for (arg, param_ty) in args.iter().zip(param_types) {
            if let Some(literal) =
                get_literal_from_ref_string_from(cx, arg).or_else(|| get_literal_from_ref_string_into(cx, arg))
                && rustc_hir_typeck::can_coerce(cx.tcx, cx.param_env, body_owner, *param_ty, str_ref)
            {
                span_lint_and_sugg(
                    cx,
                    REF_STRING_FROM_INSTEAD_OF_STR,
                    arg.span,
                    "this creates a needless String allocation, use &str instead",
                    "change this into &str",
                    format!(r#""{}""#, literal.as_str()),
                    Applicability::MaybeIncorrect,
                );
            }
        }
    }
}

fn get_literal_from_ref_string_from(cx: &LateContext<'_>, expr: &Expr<'_>) -> Option<Symbol> {
    if let ExprKind::AddrOf(BorrowKind::Ref, Mutability::Not, inner) = expr.kind
        && let ExprKind::Call(func, args) = inner.kind
        && let ExprKind::Path(qpath) = func.kind
        && let Res::Def(_, def_id) = cx.typeck_results().qpath_res(&qpath, func.hir_id)
        && cx.tcx.is_diagnostic_item(sym::from_fn, def_id)
        && args.len() == 1
        && let ExprKind::Lit(ref lit) = args[0].kind
        && let LitKind::Str(s, _) = lit.node
    {
        Some(s)
    } else {
        None
    }
}

// Check if we are calling `Into::into` where the node args match with our `From::from` signature:
// From::from signature: fn(S1) -> S2
// <S1 as Into<S2>>::into(s1), node_args=[S1, S2]
// If they do match, then it must mean that it is the blanket impl,
// which calls back into our `From::from` again (`Into` is not specializable).
// rustc's unconditional_recursion already catches calling `From::from` directly

fn get_literal_from_ref_string_into(cx: &LateContext<'_>, expr: &Expr<'_>) -> Option<Symbol> {
    println!("EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE");
    if let ExprKind::AddrOf(BorrowKind::Ref, Mutability::Not, inner) = expr.kind
        && let ExprKind::MethodCall(path, _, args, _) = inner.kind
            // Get the hir_id of the object we are calling the method on
            // Is the method to_string() ?
            && path.ident.name == sym::to_string
            // Is the method a part of the ToString trait? (i.e. not to_string() implemented
            // separately)
            && cx
                .typeck_results()
                .type_dependent_def_id(inner.hir_id)
                .opt_parent(cx)
                .is_diag_item(cx, sym::ToString)
    /*
        && let ExprKind::MethodCall(_, _, args, _) = dbg!(inner.kind)
        && dbg!({
            println!("2");
            true
        })
        && let Some(def_id) = cx.typeck_results().type_dependent_def_id(expr.hir_id)
        && dbg!({
            println!("3");
            true
        })
        && let Some(trait_def_id) = cx.tcx.trait_of_assoc(def_id)
        && dbg!({
            println!("4");
            true
        })
        && cx.tcx.is_diagnostic_item(sym::ToString, trait_def_id)
        && args.len() == 2
    */
    && dbg!({
            println!("4");
            true
        })

    && let ExprKind::Lit(ref lit) = args[0].kind
    && dbg!({
            println!("5");
            true
        })

    && let LitKind::Str(s, _) = lit.node
&& dbg!({
            println!("6");
            true
        })
    {
        Some(s)
    } else {
        None
    }

    /*
    if let Some((fn_def_id, node_args)) = dbg!(fn_def_id_with_node_args(cx, expr))
        && let [s1, s2] = **node_args
        && let (Some(s1), Some(s2)) = (s1.as_type(), s2.as_type())
        && let Some(trait_def_id) = cx.tcx.trait_of_assoc(fn_def_id)
        && cx.tcx.is_diagnostic_item(sym::Into, trait_def_id)
    // && get_impl_trait_def_id(cx, method_def_id) == cx.tcx.get_diagnostic_item(sym::From)
    // && s1 == sig.inputs()[0]
    // && s2 == sig.output()
    {
        None
    } else {
        None
    }
    */
}
