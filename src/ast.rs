use std::fmt;
use super::attribute::{Variant, Accent, LineThickness, ColumnAlign};
use crate::DisplayStyle;

/// AST node
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Number(String),
    Letter(char, Variant),
    Operator(char),
    Function(String, Option<Box<Node>>),
    Space(f32),
    Subscript(Box<Node>, Box<Node>),
    Superscript(Box<Node>, Box<Node>),
    SubSup{ target: Box<Node>, sub: Box<Node>, sup: Box<Node>},
    OverOp(char, Accent, Box<Node>),
    UnderOp(char, Accent, Box<Node>),
    Overset{over: Box<Node>, target: Box<Node>},
    Underset{under: Box<Node>, target: Box<Node>},
    Under(Box<Node>, Box<Node>),
    UnderOver { target: Box<Node>, under: Box<Node>, over: Box<Node>},
    Sqrt(Option<Box<Node>>, Box<Node>),
    Frac(Box<Node>, Box<Node>, LineThickness),
    Row(Vec<Node>),
    Fenced { open: &'static str, close: &'static str, content: Box<Node> },
    StrechedOp(bool, String),
    OtherOperator(&'static str),
    SizedParen{ size: &'static str, paren: &'static str },
    Text(String),
    Strikethrough(Box<Node>),
    Matrix(Vec<Node>, ColumnAlign),
    Piecewise(Vec<Node>),
    Ampersand,
    NewLine,
    Slashed(Box<Node>),
    Style(Option<DisplayStyle>, Box<Node>),
    Color(String, Box<Node>),
    Phantom(Box<Node>),
    Undefined(String),
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Number(number)  => write!(f, "<mn>{}</mn>", number),
            Node::Letter(letter, var) => match var {
                Variant::Italic => write!(f, "<mi>{}</mi>", letter),
                var             => write!(f, r#"<mi mathvariant="{}">{}</mi>"#, var, letter),
            },
            Node::Operator(op) => if op == &'∂' {
                write!(f, r#"<mo mathvariant="italic">∂</mo>"#)
            } else { write!(f, r#"<mo>{}</mo>"#, op) },
            Node::Function(fun, arg) => match arg {
                Some(arg) => write!(f, "<mi>{}</mi><mo>&#x2061;</mo>{}", fun, arg),
                None      => write!(f, "<mi>{}</mi>", fun),
            },
            Node::Space(space) => write!(f, r#"<mspace width="{}em"/>"#, space),
            Node::Subscript(target, b) => {
                match &**b {
                    Node::Superscript(sub,sup) =>  write!(f, "<msubsup>{}{}{}</msubsup>", target, sub, sup),
                    _ => write!(f, "<msub>{}{}</msub>", target, b),
                }
            },
            Node::Superscript(target, b) => {
                match &**b {
                    Node::Subscript(sup,sub) =>  write!(f, "<msubsup>{}{}{}</msubsup>", target, sub, sup),
                    _ => write!(f, "<msup>{}{}</msup>", target, b),
                }
            },
            Node::SubSup{target, sub, sup} => write!(f, "<msubsup>{}{}{}</msubsup>", target, sub, sup),
            Node::OverOp(op, acc, target) => write!(f, r#"<mover>{}<mo accent="{}">{}</mo></mover>"#, target, acc, op),
            Node::UnderOp(op, acc, target) => write!(f, r#"<munder>{}<mo accent="{}">{}</mo></munder>"#, target, acc, op),
            Node::Overset{over, target} => write!(f, r#"<mover>{}{}</mover>"#, target, over),
            Node::Underset{under, target} => write!(f, r#"<munder>{}{}</munder>"#, target, under),
            Node::Under(target, under) => write!(f, r#"<munder>{}{}</munder>"#, target, under),
            Node::UnderOver{target, under, over} => write!(f, r#"<munderover>{}{}{}</munderover>"#, target, under, over),
            Node::Sqrt(degree, content) => match degree {
                Some(deg) => write!(f, "<mroot>{}{}</mroot>", content, deg),
                None      => write!(f, "<msqrt>{}</msqrt>", content),
            },
            Node::Frac(num, denom, lt) => write!(f, "<mfrac{}>{}{}</mfrac>", lt, num, denom),
            Node::Row(vec) => {
                write!(f, "<mrow>")?;
                for node in vec.iter() {
                    write!(f, "{}", node)?;
                }
                write!(f, "</mrow>")
            },
            Node::Fenced{open, close, content} => {
                write!(f, r#"<mrow><mo stretchy="true" form="prefix">{}</mo>{}<mo stretchy="true" form="postfix">{}</mo></mrow>"#, open, content, close)
            },
            Node::StrechedOp(stretchy, op) => write!(f, r#"<mo stretchy="{}">{}</mo>"#, stretchy, op),
            Node::OtherOperator(op) => write!(f, "<mo>{}</mo>", op),
            Node::SizedParen{size, paren} => write!(f, r#"<mrow><mo maxsize="{0}" minsize="{0}">{1}</mro></mrow>"#, size, paren),
            Node::Slashed(node) => match &**node {
                Node::Letter(x, var) => write!(f, "<mi mathvariant=\"{}\">{}&#x0338;</mi>", var, x),
                Node::Operator(x) => write!(f, "<mo>{}&#x0338;</mo>", x),
                n => write!(f, "{}", n),
            },
            Node::Matrix(content, columnalign) => {
                let mut mathml = format!("<mtable{}><mtr><mtd>", columnalign);
                for (i, node) in content.iter().enumerate() {
                    match node {
                        Node::NewLine => {
                            mathml.push_str("</mtd></mtr>");
                            if i < content.len() {
                                mathml.push_str("<mtr><mtd>")
                            }
                        },
                        Node::Ampersand => {
                            mathml.push_str("</mtd>");
                            if i < content.len() {
                                mathml.push_str("<mtd>")
                            }
                        },
                        node => mathml.push_str(&node.to_string()),
                    }
                }
                mathml.push_str("</mtd></mtr></mtable>");
                
                write!(f, "{}", mathml)
            },
            Node::Piecewise(content) => {
                let mut mathml = "<piecewise>".to_string();
                let mut cond = String::with_capacity(32);
                let mut expr = String::with_capacity(32);
                let mut is_expr: bool = true;
                for node in content.iter() {
                    match node {
                        Node::NewLine => {
                            if !cond.is_empty() || !expr.is_empty() {
                                mathml.push_str("<piece>");
                                if !cond.is_empty() {
                                    mathml.push_str(&format!("<apply>{cond}</apply>"));
                                }
                                if !expr.is_empty() {
                                    mathml.push_str(&format!("<apply>{expr}</apply>"));
                                }
                                mathml.push_str("</piece>");
                            }
                            expr.clear();
                            cond.clear();
                            is_expr = true;
                        },
                        // Start of the condition: set is_expr to false
                        Node::Ampersand => is_expr = false,
                        // Save condition/expression nodes in a separate string since order is reversed in MathML
                        node if is_expr => expr.push_str(&node.to_string()),
                        node  => cond.push_str(&node.to_string()),
                    }
                }
                if !cond.is_empty() || !expr.is_empty() {
                    mathml.push_str("<piece>");
                    if !cond.is_empty() {
                        mathml.push_str(&format!("<apply>{cond}</apply>"));
                    }
                    if !expr.is_empty() {
                        mathml.push_str(&format!("<apply>{expr}</apply>"));
                    }
                    mathml.push_str("</piece>");
                }
                mathml.push_str("</piecewise>");
                write!(f, "{}", mathml)
            }
            Node::Text(text) => write!(f, "<mtext>{}</mtext>", text),
            Node::Strikethrough(content) => write!(f, r#"<menclose notation="updiagonalstrike">{content}</menclose>"#),
            Node::Style(display, content) => match display {
                Some(DisplayStyle::Block)  => write!(f, r#"<mstyle displaystyle="true">{}</mstyle>"#, content),
                Some(DisplayStyle::Inline) => write!(f, r#"<mstyle displaystyle="false">{}</mstyle>"#, content),
                None => write!(f, "<mstyle>{}</mstyle>", content),
            },
            Node::Color(color, content) => write!(f, r#"<mstyle style="color: {color};">{content}</mstyle>"#),
            Node::Phantom(content) => write!(f, r#"<mphantom>{content}</mphantom>"#),
            node => write!(f, "<merror>[PARSE ERROR: {:?}]</merror>", node),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::attribute::Variant;
    use super::Node;

    #[test]
    fn node_display() {
        let problems = vec![
            (Node::Number("3.14".to_owned()), "<mn>3.14</mn>"),
            (Node::Letter('x', Variant::Italic), "<mi>x</mi>"),
            (Node::Letter('α', Variant::Italic), "<mi>α</mi>"),
            (Node::Letter('あ', Variant::Normal), r#"<mi mathvariant="normal">あ</mi>"#),
            (
                Node::Row(vec![ Node::Operator('+'), Node::Operator('-') ]), 
                r"<mrow><mo>+</mo><mo>-</mo></mrow>"
            ),
        ];
        for (problem, answer) in problems.iter() {
            assert_eq!(&format!("{}", problem), answer);
        }
    }
}
