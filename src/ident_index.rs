use rnix::ast::HasEntry;
use rowan::ast::AstNode;

/// Returns every identifier in `expr` that could refer to a package, with its byte offset.
pub fn identifiers(expr: &rnix::ast::Expr) -> Vec<(String, usize)> {
    let mut idents = vec![];
    expr.collect_identifiers(&mut idents);
    idents
}

trait CollectIdentifiers {
    fn collect_identifiers(&self, result: &mut Vec<(String, usize)>);
}

impl CollectIdentifiers for rnix::ast::Ident {
    fn collect_identifiers(&self, result: &mut Vec<(String, usize)>) {
        let position = self.ident_token().unwrap().text_range().start();
        let text = self.ident_token().unwrap().text().to_string();
        result.push((text, usize::from(position)));
    }
}

impl CollectIdentifiers for rnix::ast::Str {
    fn collect_identifiers(&self, result: &mut Vec<(String, usize)>) {
        for p in self.normalized_parts() {
            match p {
                rnix::ast::InterpolPart::Literal(s) => {
                    let position = self.syntax().text_range().start();
                    result.push((s, usize::from(position)));
                }
                rnix::ast::InterpolPart::Interpolation(x) => {
                    x.expr().unwrap().collect_identifiers(result);
                }
            }
        }
    }
}

impl CollectIdentifiers for rnix::ast::Attr {
    fn collect_identifiers(&self, result: &mut Vec<(String, usize)>) {
        use rnix::ast::Attr::*;
        match self {
            Ident(z) => z.collect_identifiers(result),
            Dynamic(z) => z.expr().unwrap().collect_identifiers(result),
            Str(z) => z.collect_identifiers(result),
        }
    }
}

impl CollectIdentifiers for rnix::ast::Expr {
    fn collect_identifiers(&self, result: &mut Vec<(String, usize)>) {
        use rnix::ast::Expr::*;
        match self {
            Apply(x) => {
                x.lambda().unwrap().collect_identifiers(result);
                x.argument().unwrap().collect_identifiers(result);
            }
            Assert(x) => {
                x.condition().unwrap().collect_identifiers(result);
                x.body().unwrap().collect_identifiers(result);
            }
            Error(x) => {
                panic!("What is this Error: {:?}", x)
            }
            IfElse(x) => {
                x.condition().unwrap().collect_identifiers(result);
                x.body().unwrap().collect_identifiers(result);
                x.else_body().unwrap().collect_identifiers(result);
            }
            Select(x) => {
                x.expr().unwrap().collect_identifiers(result);
                for a in x.attrpath().unwrap().attrs() {
                    a.collect_identifiers(result);
                }
            }
            Str(x) => {
                x.collect_identifiers(result);
            }
            PathAbs(_x) => {}
            PathRel(_x) => {}
            PathHome(_x) => {}
            PathSearch(_x) => {}
            Literal(_x) => {}
            Lambda(x) => {
                use rnix::ast::Param::*;
                match x.param().unwrap() {
                    Pattern(pat) => {
                        for e in pat.pat_entries() {
                            e.ident().unwrap().collect_identifiers(result);
                            if let Some(d) = e.default() {
                                d.collect_identifiers(result)
                            }
                        }
                    }
                    IdentParam(_b) => {}
                }
                x.body().unwrap().collect_identifiers(result);
            }
            LegacyLet(_x) => {
                panic!("Don't use let body..")
            }
            LetIn(x) => {
                collect_identifiers_from_entries(x, true, result);
                x.body().unwrap().collect_identifiers(result);
            }
            List(x) => {
                for i in x.items() {
                    i.collect_identifiers(result);
                }
            }
            BinOp(x) => {
                x.lhs().unwrap().collect_identifiers(result);
                x.rhs().unwrap().collect_identifiers(result);
            }
            Paren(x) => {
                x.expr().unwrap().collect_identifiers(result);
            }
            Root(_x) => {
                panic!("What is root doing here");
            }
            AttrSet(x) => {
                collect_identifiers_from_entries(x, false, result);
            }
            UnaryOp(x) => {
                x.expr().unwrap().collect_identifiers(result);
            }
            Ident(x) => {
                // TODO: Which identifiers could even refer to packages?
                // Within with statements for sure, but we can check for that
                x.collect_identifiers(result);
            }
            With(x) => {
                x.namespace().unwrap().collect_identifiers(result);
                x.body().unwrap().collect_identifiers(result);
            }
            HasAttr(x) => {
                x.expr().unwrap().collect_identifiers(result);
                for a in x.attrpath().unwrap().attrs() {
                    a.collect_identifiers(result);
                }
            }
            CurPos(_x) => {}
        }
    }
}

fn collect_identifiers_from_entries<E: HasEntry>(
    e: &E,
    _skip_first: bool,
    result: &mut Vec<(String, usize)>,
) {
    for e in e.entries() {
        use rnix::ast::Entry::*;
        match e {
            Inherit(y) => {
                if let Some(f) = y.from() {
                    f.expr().unwrap().collect_identifiers(result)
                }
                for a in y.attrs() {
                    a.collect_identifiers(result);
                }
            }
            AttrpathValue(y) => {
                // This code would make `a.b.c = d` a dependency on identifiers a, b and c
                // While this is possible, it's very unlikely, and causes many false positives like
                // in all-packages.nix and similar files
                //for z in y
                //    .attrpath()
                //    .unwrap()
                //    .attrs()
                //    .skip(if skip_first { 1 } else { 0 })
                //{
                //    z.collect_identifiers(result);
                //}
                y.value().unwrap().collect_identifiers(result);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    #[test]
    fn basic() -> anyhow::Result<()> {
        let contents = indoc! {r#"
            # TODO: Create an expression that tests all code paths
            null
        "#};

        let expr = rnix::Root::parse(&contents)
            .ok()?
            .expr()
            .ok_or(anyhow::format_err!("No expr"))?;

        assert_eq!(identifiers(&expr), vec![("null".to_string(), 55)]);
        Ok(())
    }
}
