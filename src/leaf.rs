use rnix::ast::HasEntry;
use std::collections::HashSet;

pub fn pprefs(expr: rnix::ast::Expr) -> HashSet<String> {
    let mut idents = HashSet::new();
    expr.pprefs(&mut idents);
    idents
}

trait CanReferencePackages {
    fn pprefs(&self, result: &mut HashSet<String>);
}

impl CanReferencePackages for str {
    fn pprefs(&self, result: &mut HashSet<String>) {
        // Even though many strings/identifiers usually aren't used for packages, we don't want to
        // miss any, so don't limit it
        result.insert(self.to_string());
    }
}

impl CanReferencePackages for rnix::ast::Ident {
    fn pprefs(&self, result: &mut HashSet<String>) {
        self.ident_token().unwrap().text().pprefs(result);
    }
}

impl CanReferencePackages for rnix::ast::Str {
    fn pprefs(&self, result: &mut HashSet<String>) {
        for p in self.normalized_parts() {
            match p {
                rnix::ast::InterpolPart::Literal(s) => {
                    s.pprefs(result);
                }
                rnix::ast::InterpolPart::Interpolation(x) => {
                    x.expr().unwrap().pprefs(result);
                }
            }
        }
    }
}

impl CanReferencePackages for rnix::ast::Attr {
    fn pprefs(&self, result: &mut HashSet<String>) {
        use rnix::ast::Attr::*;
        match self {
            Ident(z) => z.pprefs(result),
            Dynamic(z) => z.expr().unwrap().pprefs(result),
            Str(z) => z.pprefs(result),
        }
    }
}

impl CanReferencePackages for rnix::ast::Expr {
    fn pprefs(&self, result: &mut HashSet<String>) {
        use rnix::ast::Expr::*;
        match self {
            Apply(x) => {
                x.lambda().unwrap().pprefs(result);
                x.argument().unwrap().pprefs(result);
            }
            Assert(x) => {
                x.condition().unwrap().pprefs(result);
            }
            Error(x) => {
                panic!("What is this Error: {:?}", x)
            }
            IfElse(x) => {
                x.condition().unwrap().pprefs(result);
                x.body().unwrap().pprefs(result);
                x.else_body().unwrap().pprefs(result);
            }
            Select(x) => {
                x.expr().unwrap().pprefs(result);
                for a in x.attrpath().unwrap().attrs() {
                    a.pprefs(result);
                }
            }
            Str(x) => {
                x.pprefs(result);
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
                            e.ident().unwrap().pprefs(result);
                            if let Some(d) = e.default() {
                                d.pprefs(result)
                            }
                        }
                    }
                    IdentParam(_b) => {}
                }
                x.body().unwrap().pprefs(result);
            }
            LegacyLet(_x) => {
                panic!("Don't use let body..")
            }
            LetIn(x) => {
                pprefs_from_entries(x, true, result);
                x.body().unwrap().pprefs(result);
            }
            List(x) => {
                for i in x.items() {
                    i.pprefs(result);
                }
            }
            BinOp(x) => {
                x.lhs().unwrap().pprefs(result);
                x.rhs().unwrap().pprefs(result);
            }
            Paren(x) => {
                x.expr().unwrap().pprefs(result);
            }
            Root(_x) => {
                panic!("What is root doing here");
            }
            AttrSet(x) => {
                pprefs_from_entries(x, false, result);
            }
            UnaryOp(x) => {
                x.expr().unwrap().pprefs(result);
            }
            Ident(x) => {
                // TODO: Which identifiers could even refer to packages?
                // Within with statements for sure, but we can check for that
                x.pprefs(result);
            }
            With(x) => {
                x.namespace().unwrap().pprefs(result);
                x.body().unwrap().pprefs(result);
            }
            HasAttr(x) => {
                x.expr().unwrap().pprefs(result);
                for a in x.attrpath().unwrap().attrs() {
                    a.pprefs(result);
                }
            }
            CurPos(_x) => {}
        }
    }
}

fn pprefs_from_entries<E: HasEntry>(e: &E, skip_first: bool, result: &mut HashSet<String>) {
    for e in e.entries() {
        use rnix::ast::Entry::*;
        match e {
            Inherit(y) => {
                if let Some(f) = y.from() {
                    f.expr().unwrap().pprefs(result)
                }
                for a in y.attrs() {
                    a.pprefs(result);
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
                //    z.pprefs(result);
                //}
                y.value().unwrap().pprefs(result);
            }
        }
    }
}
