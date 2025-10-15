use super::*;

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::comment))]
pub enum Comment {
    Line(CommentLine),
    Markdown(CommentMarkdown),
    Wild(CommentWild),
}

impl ToString for Comment {
    fn to_string(&self) -> String {
        match self {
            Comment::Line(line) => line.value.clone(),
            Comment::Markdown(md) => md.value.clone(),
            Comment::Wild(wild) => wild.value.clone(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, FromPest, Clone)]
#[pest_ast(rule(Rule::comment_line))]
pub struct CommentLine {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub value: String,
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::comment_md))]
pub struct CommentMarkdown {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub value: String,
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::comment_wild))]
pub struct CommentWild {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub value: String,
}
