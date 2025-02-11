#[cfg(debug_assertions)]
#[allow(unused_imports)]
use log::{debug, error, info};
use {
    crate::pmwiki::IPmwiki,
    nom::{
        branch::alt,
        bytes::complete::{
            // take_until1,
            // take_while, // take while function result is true
            // take_while1,
            // take_while_m_n, // longest m<=len<=n
            is_not,
            tag, // exact match
            // tag_no_case, // same, case insensitive
            // take, // blindly take by argument as count
            // take_till, // take before given function over input returns true
            // take_till1, // same, error on blank result
            take_until, // take before given pattern equals from remaining input
        },
        character::complete::{
            char,
            // not_line_ending, line_ending,
            // none_of,
            one_of,
            space0,
            // space1,
            // multispace0, multispace1,
        },
        combinator::{
            // consumed, // returns tuple (consumed input, parser output) as result
            eof,
            map, // apply function to the result of parser
            // rest_len, // return length of all remaining input
            peek,
            // not, // success when parser fails
            // cond, // take a bool and run parser when it's a true
            // all_consuming, // succeed when given parser remains no rest input
            recognize, // return consumed input by parser when its succeed
            rest,      // return all remaining input
            success,
            // map_opt, // same, returns Option<_>
            // map_res, // same, returns Result<_>
            // map_parser, // parser after parser
            value,  // return first argument as parser result if parser succeed
            verify, // same, if given verify function returns true over parser result
        },
        // InputLength,
        // InputTakeAtPosition,
        // Parser,
        // Err,
        error::{
            // Error,
            ErrorKind,
            ParseError,
        },
        multi::{
            // fill, // fill given slice or fail
            // fold_many0, // apply until fail
            // fold_many1, // same, does not allow empty result
            // fold_many_m_n, // m<=count<=n
            // length_count, // result of first parser is the count to repeat second parser
            // length_data, // result of first parser is the count to take
            // length_value, // result of first parser is the count to take and it's the input to second parser
            // many0_count, // repeat parser until fails and return the count of successful iteration
            // many1_count, // same, fail on zero
            many0,
            // many1,
            many_m_n,
            // many_till,
            separated_list0,
            separated_list1, // two parsers for list
        },
        sequence::{
            delimited, // "(a)" => "a"
            pair,      // "ab" => ("a", "b")
            preceded,  // "(a" => "a"
            // tuple, // same, up to 21 elements
            separated_pair, // "a,b" => ("a", "b")
            terminated,     // "a)" => "a"
        },
        IResult,
    },
};

fn bold(input: &str) -> IResult<&str, IPmwiki<'_>> {
    map(
        delimited(
            tag("'''"),
            collect_while_parser_fail_or0(
                alt((value(true, peek(tag("'''"))), value(true, peek(char('\n'))))),
                italic,
                text,
            ),
            tag("'''"),
        ),
        IPmwiki::Bold,
    )(input)
}
fn italic(input: &str) -> IResult<&str, IPmwiki<'_>> {
    map(
        delimited(
            tag("''"),
            collect_while_parser_fail_or0(
                alt((value(true, peek(tag("''"))), value(true, peek(char('\n'))))),
                alt((bold, link)),
                text,
            ),
            tag("''"),
        ),
        IPmwiki::Italic,
    )(input)
}

fn text_style(input: &str) -> IResult<&str, IPmwiki<'_>> {
    alt((
        value(IPmwiki::ForceLinebreak, tag("\\\\")),
        bold,
        italic,
        // #[cfg(feature="font-color")]
        // font_color,
    ))(input)
}

fn list_head_char(input: &str) -> IResult<&str, char> {
    one_of("*#")(input)
}
fn list(input: &str) -> IResult<&str, IPmwiki> {
    let input = input.trim_start();
    let (_, head) = list_head_char(input)?;
    let (r, lines) = separated_list0(
        char('\n'),
        recognize(separated_pair(
            preceded(space0, preceded(char(head), many0(list_head_char))),
            char(' '),
            alt((is_not("\n"), success(""))),
        )),
    )(input)?;

    // debug!("list recognized : {:?}", lines);
    if lines.is_empty() {
        return Err(nom::Err::Error(nom::error::Error {
            input,
            code: ErrorKind::Tag,
        }));
    }
    let v = _list(&input[..1])(lines)?;

    Ok((r, v))
}

fn _list<'a>(
    head_tag: &'a str,
) -> impl FnMut(Vec<&'a str>) -> Result<IPmwiki, nom::Err<nom::error::Error<&'a str>>> {
    move |input: Vec<&str>| {
        let head_space = format!("{} ", head_tag);
        let mut rst = vec![];
        let mut to_skip = 0;
        for i in 0..input.len() {
            if to_skip > 0 {
                to_skip -= 1;
                continue;
            }
            let line = input[i].trim_start();
            // debug!("list line : {}", line);
            if line.starts_with(&head_space) {
                // sibling
                if let Ok((_, v)) = map(collect_while_parser_fail0(lit, text), IPmwiki::ListItem)(
                    &line[head_space.len()..],
                ) {
                    rst.push(v);
                }
            } else {
                // child
                let child_head_tag = &line[..head_space.len()];
                let mut child_lines = vec![line];
                for line in &input[(i + 1)..] {
                    if line.starts_with(&child_head_tag) {
                        child_lines.push(*line);
                    } else {
                        break;
                    }
                }
                to_skip = child_lines.len() - 1;
                // debug!("child lines : {:?}, skip : {}", child_lines, to_skip);
                if let Ok(v) = _list(child_head_tag)(child_lines) {
                    rst.push(v);
                }
            }
        }
        Ok(if head_tag.ends_with('*') {
            IPmwiki::BulletList(rst)
        } else {
            IPmwiki::NumberedList(rst)
        })
    }
}

fn link(input: &str) -> IResult<&str, IPmwiki> {
    map(
        delimited(
            tag("[["),
            alt((
                separated_pair(is_not("[]|\n"), tag("|"), is_not("[]\n")),
                map(is_not("[]\n"), |v: &str| (v, v)),
            )),
            tag("]]"),
        ),
        |(link, label)| IPmwiki::Link(link, label),
    )(input)
}
fn image(input: &str) -> IResult<&str, IPmwiki> {
    map(
        delimited(
            tag("[{"),
            alt((
                separated_pair(is_not("{}|\n"), tag("|"), is_not("{}\n")),
                map(is_not("{}\n"), |src: &str| (src, "")),
            )),
            tag("}]"),
        ),
        |(src, label)| IPmwiki::Image(src, label),
    )(input)
}

fn lit(input: &str) -> IResult<&str, IPmwiki> {
    alt((link, image, text_style))(input)
}
fn dlit(input: &str) -> IResult<&str, IPmwiki> {
    alt((dont_format, lit))(input)
}
fn text(input: &str) -> IResult<&str, IPmwiki> {
    map(verify(rest, |s: &str| !s.is_empty()), |s: &str| {
        IPmwiki::Text(s)
    })(input)
}
fn take_dlit_text_until0<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Vec<IPmwiki>> {
    collect_opt_pair0(take_while_parser_fail(dlit, text))
}
fn take_dlit_text_until_peek_char0<'a>(
    until_char: char,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<IPmwiki>> {
    collect_opt_pair0(take_while_parser_fail_or(
        value(true, peek(char(until_char))),
        dlit,
        text,
    ))
}

fn take_while_parser_fail<'a>(
    mut parser: impl FnMut(&'a str) -> IResult<&'a str, IPmwiki<'a>>,
    mut fail_parser: impl FnMut(&'a str) -> IResult<&'a str, IPmwiki<'a>>,
) -> impl FnMut(&'a str) -> IResult<&'a str, (Option<IPmwiki<'a>>, Option<IPmwiki<'a>>)> {
    move |input: &str| {
        // debug!("take_while_parser_fail input : {}", input);
        // let mut i = input;
        let mut l = 0;
        for (i, c) in input.char_indices().by_ref() {
            // debug!("take_while_parser_fail i : {}", i);
            if let Ok((r, v)) = parser(&input[l..]) {
                return Ok((
                    r,
                    (
                        if l > 0 {
                            if let Ok((_, v)) = fail_parser(&input[..l]) {
                                Some(v)
                            } else {
                                None
                            }
                        } else {
                            None
                        },
                        Some(v),
                    ),
                ));
            } else {
                l = i + c.len_utf8();
            }
        }
        if l > 0 {
            if let Ok((_, f)) = fail_parser(&input[..l]) {
                Ok((&input[input.len()..], (Some(f), None)))
            } else {
                Err(nom::Err::Error(ParseError::from_error_kind(
                    input,
                    ErrorKind::Tag,
                )))
            }
        } else {
            Err(nom::Err::Error(ParseError::from_error_kind(
                input,
                ErrorKind::Eof,
            )))
        }
    }
}

fn collect_opt_pair0<'a>(
    parser: impl FnMut(&'a str) -> IResult<&'a str, (Option<IPmwiki<'a>>, Option<IPmwiki<'a>>)>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<IPmwiki<'a>>> {
    alt((collect_opt_pair1(parser), success(vec![])))
}
fn collect_opt_pair1<'a>(
    mut parser: impl FnMut(&'a str) -> IResult<&'a str, (Option<IPmwiki<'a>>, Option<IPmwiki<'a>>)>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<IPmwiki<'a>>> {
    move |input: &str| {
        let mut rst = vec![];
        let mut i = input;
        // debug!("collect_opt_pair input : {}", i);
        while let Ok((r, t)) = parser(i) {
            // debug!("collect_opt_pair i : {}, r: {}", i, r);
            i = r;
            match t {
                (Some(a), Some(b)) => {
                    rst.push(a);
                    rst.push(b);
                }
                (None, Some(b)) => {
                    rst.push(b);
                }
                (Some(a), None) => {
                    rst.push(a);
                    break;
                }
                _ => {
                    break;
                }
            }
        }
        // debug!("collect_opt_pair rst : {:?}", rst);
        if rst.is_empty() {
            Err(nom::Err::Error(ParseError::from_error_kind(
                input,
                ErrorKind::Eof,
            )))
        } else {
            Ok((i, rst))
        }
    }
}

fn take_while_parser_fail_or<'a>(
    mut term_parser: impl FnMut(&'a str) -> IResult<&'a str, bool>,
    mut parser: impl FnMut(&'a str) -> IResult<&'a str, IPmwiki<'a>>,
    mut fail_parser: impl FnMut(&'a str) -> IResult<&'a str, IPmwiki<'a>>,
) -> impl FnMut(&'a str) -> IResult<&'a str, (Option<IPmwiki<'a>>, Option<IPmwiki<'a>>)> {
    move |input: &str| {
        let mut l = 0;
        for (i, c) in input.char_indices().by_ref() {
            if let Ok((i, true)) = term_parser(&input[l..]) {
                return Ok((i, (fail_parser(&input[..l]).ok().map(|(_, v)| v), None)));
            } else if let Ok((r, v)) = parser(&input[l..]) {
                return Ok((r, (fail_parser(&input[..l]).ok().map(|(_, v)| v), Some(v))));
            } else {
                l = i + c.len_utf8();
            }
        }
        if l > 0 {
            if let Ok((_, f)) = fail_parser(&input[..l]) {
                Ok(("", (Some(f), None)))
            } else {
                Err(nom::Err::Error(ParseError::from_error_kind(
                    input,
                    ErrorKind::Tag,
                )))
            }
        } else {
            Err(nom::Err::Error(ParseError::from_error_kind(
                input,
                ErrorKind::Eof,
            )))
        }
    }
}

// fn take_while_parser_fail_or_peek_tag<'a>(
//     term_tag: &'static str,
//     parser: impl FnMut(&'a str) -> IResult<&'a str, IPmwiki<'a>>,
//     fail_parser: impl FnMut(&'a str) -> IResult<&'a str, IPmwiki<'a>>,
// ) -> impl FnMut(&'a str) -> IResult<&'a str, (Option<IPmwiki<'a>>, Option<IPmwiki<'a>>)> {
//     take_while_parser_fail_or(value(true, peek(tag(term_tag))), parser, fail_parser)
// }

fn collect_while_parser_fail_or0<'a>(
    term_parser: impl FnMut(&'a str) -> IResult<&'a str, bool>,
    parser: impl FnMut(&'a str) -> IResult<&'a str, IPmwiki<'a>>,
    fail_parser: impl FnMut(&'a str) -> IResult<&'a str, IPmwiki<'a>>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<IPmwiki<'a>>> {
    collect_opt_pair0(take_while_parser_fail_or(term_parser, parser, fail_parser))
}

fn collect_while_parser_fail0<'a>(
    parser: impl FnMut(&'a str) -> IResult<&'a str, IPmwiki<'a>>,
    fail_parser: impl FnMut(&'a str) -> IResult<&'a str, IPmwiki<'a>>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<IPmwiki<'a>>> {
    collect_opt_pair0(take_while_parser_fail(parser, fail_parser))
}

// #[cfg(feature="link-button")]
// fn link_button(input: &str) -> IResult<&str, IPmwiki> {
//   map(delimited(tag("[{"), alt((
//     separated_pair(is_not("|]}\n"), tag("|"), is_not("|]}\n")),
//     map(is_not("]}\n"), |label: &str| -> (&str, &str) { (label, label) }),
//   )), tag("]}")), |(link, label)| IPmwiki::LinkButton(label, link, ""))(input)
// }
// #[cfg(feature="font-color")]
// fn font_color(input: &str) -> IResult<&str, IPmwiki> {
//   map(delimited(tag("[{"), alt((
//     separated_pair(is_not("|]}\n"), tag("|"), is_not("|]}\n")),
//     map(is_not("]}\n"), |label: &str| -> (&str, &str) { (label, label) }),
//   )), tag("]}")), |(link, label)| IPmwiki::Link(label, link))(input)
// }

fn heading(input: &str) -> IResult<&str, IPmwiki> {
    map(
        separated_pair(
            map(many_m_n(2, 7, char('!')), |s| s.len()-1),
            char(' '),
            alt((take_until("\n"), rest)),
        ),
        |(level, body): (usize, &str)| {
            IPmwiki::Heading(
                level as u8,
                if let Ok((_, v)) = take_dlit_text_until0()({
                    let body = body.trim_end();
                    if body.ends_with(&input[..level]) {
                        // ignore ending space and '='s same as its opening
                        body[..body.len() - level].trim_end()
                    } else {
                        body
                    }
                }) {
                    v
                } else {
                    vec![]
                },
            )
        },
    )(input)
}
fn dont_format(input: &str) -> IResult<&str, IPmwiki> {
    map(
        delimited(tag("[="), take_until("=]"), tag("=]")),
        IPmwiki::DontFormat,
    )(input)
}

fn table_header_cell_inner(input: &str) -> IResult<&str, IPmwiki> {
    map(
        take_dlit_text_until_peek_char0('|'),
        IPmwiki::TableHeaderCell,
    )(input)
}
fn table_cell_inner(input: &str) -> IResult<&str, IPmwiki> {
    map(take_dlit_text_until_peek_char0('|'), IPmwiki::TableCell)(input)
}
fn table_header_row(input: &str) -> IResult<&str, IPmwiki> {
    let (left, line) = preceded(
        tag("|="),
        map(
            alt((
                terminated(
                    verify(map(is_not("\n"), |s: &str| s.trim_end()), |s: &str| {
                        s.ends_with('|')
                    }),
                    char('\n'),
                ),
                verify(rest, |s: &str| s.len() > 1 && s.ends_with('|')),
            )),
            |s: &str| &s[..(s.len() - 1)],
        ),
    )(input)?;
    // debug!("table_header_row line : {}", line);

    let (_, rst) = map(separated_list1(tag("|="), table_header_cell_inner), |v| {
        IPmwiki::TableHeaderRow(v)
    })(line)?;
    Ok((left, rst))
}
fn table_cell_row(input: &str) -> IResult<&str, IPmwiki> {
    let (left, line) = preceded(
        char('|'),
        map(
            alt((
                verify(map(is_not("\n"), |s: &str| s.trim_end()), |s: &str| {
                    s.ends_with('|')
                }),
                verify(map(rest, |s: &str| s.trim_end()), |s: &str| {
                    s.len() > 1 && s.ends_with('|')
                }),
            )),
            |s: &str| &s[..(s.len() - 1)],
        ),
    )(input)?;
    // debug!("table_cell_row : {}", line);
    let (_, rst) = map(separated_list1(tag("|"), table_cell_inner), |v| {
        IPmwiki::TableRow(v)
    })(line)?;
    Ok((left, rst))
}
fn table(input: &str) -> IResult<&str, IPmwiki> {
    let mut rst = vec![];
    let mut rest = input;
    // debug!("table input : {}", input);
    if let Ok((rest, body)) = if let Ok((r, head)) = table_header_row(input) {
        // debug!("table head : {:?}", head);
        rst.push(head);
        rest = r;
        separated_list0(char('\n'), table_cell_row)(r)
    } else {
        // debug!("table head is empty");
        separated_list1(char('\n'), table_cell_row)(input)
    } {
        // debug!("body : {:?}", body);
        return Ok((rest, IPmwiki::Table([rst, body].concat())));
    } else if !rst.is_empty() {
        // debug!("no body found, result : {:?}", rst);
        return Ok((rest, IPmwiki::Table(rst)));
    };
    // debug!("table : {:?}", rst);
    if rst.is_empty() {
        Err(nom::Err::Error(nom::error::Error {
            input,
            code: ErrorKind::Tag,
        }))
    } else {
        Ok((rest, IPmwiki::Table(rst)))
    }
}

// #[cfg(feature="fold")]
// fn fold(input: &str) -> IResult<&str, IPmwiki> {
//   map(map_parser(tag("---<"), rest), |rest| IPmwiki::Fold(rest))(input)
// }

fn line(input: &str) -> IResult<&str, IPmwiki> {
    map(
        collect_while_parser_fail_or0(
            alt((
                value(true, pair(char('\n'), peek(char('\n')))),
                value(true, peek(preceded(char('\n'), pmwiki_inner))),
            )),
            dlit,
            text,
        ),
        IPmwiki::Line,
    )(input)
}
fn pmwiki_inner(input: &str) -> IResult<&str, IPmwiki> {
    alt((
        value(
            IPmwiki::HorizontalLine,
            terminated(tag("----"), alt((peek(tag("\n")), eof))),
        ),
        heading,
        dont_format,
        table,
        list,
    ))(input)
}

pub fn try_pmwikis(input: &str) -> IResult<&str, Vec<IPmwiki>> {
    separated_list0(char('\n'), alt((pmwiki_inner, line)))(input)
}

pub fn pmwikis(input: &str) -> Vec<IPmwiki> {
    if let Ok((_, v)) = try_pmwikis(input) {
        v
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pmwiki::IPmwiki;

    fn init() {
        let _ =
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
                .is_test(true)
                .try_init();
    }

    #[test]
    fn text_tests() {
        init();
        use IPmwiki::*;
        assert_eq!(pmwikis("ab1"), vec![Line(vec![Text("ab1")])]);
    }

    #[test]
    fn text_style_tests() {
        init();
        use IPmwiki::*;
        assert_eq!(try_pmwikis("t"), Ok(("", vec![Line(vec![Text("t")])])));

        assert_eq!(
            try_pmwikis("'''b'''"),
            Ok(("", vec![Line(vec![Bold(vec![Text("b")])])]))
        );

        assert_eq!(
            try_pmwikis("''i''"),
            Ok(("", vec![Line(vec![Italic(vec![Text("i")])])]))
        );

        assert_eq!(
            try_pmwikis("a'''b'''''c''d"),
            Ok((
                "",
                vec![Line(vec![
                    Text("a"),
                    Bold(vec![Text("b")]),
                    Italic(vec![Text("c")]),
                    Text("d")
                ])]
            ))
        );
        assert_eq!(
            try_pmwikis("'''a''b''c'''"),
            Ok((
                "",
                vec![Line(vec![Bold(vec![
                    Text("a"),
                    Italic(vec![Text("b")]),
                    Text("c")
                ])])]
            ))
        );
    }

    #[test]
    fn linebreak_tests() {
        init();
        use IPmwiki::*;
        assert_eq!(
            try_pmwikis("a\\\\b"),
            Ok(("", vec![Line(vec![Text("a"), ForceLinebreak, Text("b"),])]))
        );

        assert_eq!(pmwikis("a\\b"), vec![Line(vec![Text("a\\b")])]);

        assert_eq!(
            try_pmwikis("a\nb\n\nc"),
            Ok(("", vec![Line(vec![Text("a\nb")]), Line(vec![Text("c")])]))
        );
    }

    #[test]
    fn list_tests() {
        init();
        use IPmwiki::*;
        assert_eq!(list("* "), Ok(("", BulletList(vec![ListItem(vec![])]))));
        assert_eq!(
            list("* a"),
            Ok(("", BulletList(vec![ListItem(vec![Text("a")])])))
        );
        assert_eq!(
            list("** b"),
            Ok((
                "",
                BulletList(vec![BulletList(vec![ListItem(vec![Text("b")])])])
            ))
        );
        assert_eq!(
            list("*** c"),
            Ok((
                "",
                BulletList(vec![BulletList(vec![BulletList(vec![ListItem(vec![
                    Text("c")
                ])])])])
            ))
        );
        assert_eq!(
            list(
                "* a
** b"
            ),
            Ok((
                "",
                BulletList(vec![
                    ListItem(vec![Text("a")]),
                    BulletList(vec![ListItem(vec![Text("b")])]),
                ])
            ))
        );
        assert_eq!(
            list(
                "* a
** b
** c"
            ),
            Ok((
                "",
                BulletList(vec![
                    ListItem(vec![Text("a")]),
                    BulletList(vec![ListItem(vec![Text("b")]), ListItem(vec![Text("c")]),])
                ])
            ))
        );
        assert_eq!(
            list(
                "* a
** ab
* b
** ba"
            ),
            Ok((
                "",
                BulletList(vec![
                    ListItem(vec![Text("a")]),
                    BulletList(vec![ListItem(vec![Text("ab")])]),
                    ListItem(vec![Text("b")]),
                    BulletList(vec![ListItem(vec![Text("ba")])])
                ])
            ))
        );
        assert_eq!(
            list(
                "* [[a]]
** ''b''
** '''c'''"
            ),
            Ok((
                "",
                BulletList(vec![
                    ListItem(vec![Link("a", "a")]),
                    BulletList(vec![
                        ListItem(vec![Italic(vec![Text("b")])]),
                        ListItem(vec![Bold(vec![Text("c")])]),
                    ]),
                ])
            ))
        );
        assert_eq!(
            list(
                "* a
** aa
** ab
* b
** ba"
            ),
            Ok((
                "",
                BulletList(vec![
                    ListItem(vec![Text("a")]),
                    BulletList(vec![ListItem(vec![Text("aa")]), ListItem(vec![Text("ab")]),]),
                    ListItem(vec![Text("b")]),
                    BulletList(vec![ListItem(vec![Text("ba")])]),
                ])
            ))
        );
        assert_eq!(
            pmwikis(
                "* a
** aa
** ab
* b
*# b1"
            ),
            vec![BulletList(vec![
                ListItem(vec![Text("a")]),
                BulletList(vec![ListItem(vec![Text("aa")]), ListItem(vec![Text("ab")]),]),
                ListItem(vec![Text("b")]),
                NumberedList(vec![ListItem(vec![Text("b1")])])
            ])]
        );
        assert_eq!(
            try_pmwikis(
                "* a
*# a1
*# a2
*## a31
#### 1111
#### 1112
### 112
##* 11a
##* 11b
##*# 11c1
##*# 11c2
* b"
            ),
            Ok((
                "",
                vec![
                    BulletList(vec![
                        ListItem(vec![Text("a")]),
                        NumberedList(vec![
                            ListItem(vec![Text("a1")]),
                            ListItem(vec![Text("a2")]),
                            NumberedList(vec![ListItem(vec![Text("a31")])])
                        ])
                    ]),
                    NumberedList(vec![NumberedList(vec![
                        NumberedList(vec![
                            NumberedList(vec![
                                ListItem(vec![Text("1111")]),
                                ListItem(vec![Text("1112")])
                            ]),
                            ListItem(vec![Text("112")])
                        ]),
                        BulletList(vec![
                            ListItem(vec![Text("11a")]),
                            ListItem(vec![Text("11b")]),
                            NumberedList(vec![
                                ListItem(vec![Text("11c1")]),
                                ListItem(vec![Text("11c2")])
                            ])
                        ])
                    ])]),
                    BulletList(vec![ListItem(vec![Text("b")])])
                ]
            ))
        );
        assert_eq!(
            list(" * a"),
            Ok(("", BulletList(vec![ListItem(vec![Text("a")])])))
        );
        assert_eq!(
            list("  * a"),
            Ok(("", BulletList(vec![ListItem(vec![Text("a")])])))
        );
    }
    #[test]
    fn parser_tests() {
        init();
        use IPmwiki::*;
        assert_eq!(
            take_while_parser_fail(lit, text)("a b '''a'''"),
            Ok(("", (Some(Text("a b ")), Some(Bold(vec![Text("a")])),)))
        );
        assert_eq!(
            collect_opt_pair1(take_while_parser_fail(lit, text))("a b '''a'''"),
            Ok(("", vec![Text("a b "), Bold(vec![Text("a")]),]))
        );
        assert_eq!(
            collect_opt_pair1(take_while_parser_fail(pmwiki_inner, text))("a\n!! b"),
            Ok(("", vec![Text("a\n"), Heading(1, vec![Text("b")]),]))
        );
        assert_eq!(
            collect_opt_pair1(take_while_parser_fail(lit, text))("a b '''a'''"),
            Ok(("", vec![Text("a b "), Bold(vec![Text("a")]),]))
        );
        assert_eq!(
            collect_opt_pair1(take_while_parser_fail(lit, text))(
                "[[a|b]] ''Live'' Editor ([[c|d]])"
            ),
            Ok((
                "",
                vec![
                    // Line(vec![
                    Link("a", "b"),
                    Text(" "),
                    Italic(vec![Text("Live")]),
                    Text(" Editor ("),
                    Link("c", "d"),
                    Text(")"),
                    // ])
                ]
            ))
        );
    }

    #[test]
    fn heading_tests() {
        init();
        use IPmwiki::*;
        assert_eq!(heading("!! "), Ok(("", Heading(1, vec![]))));
        assert_eq!(heading("!! a"), Ok(("", Heading(1, vec![Text("a")]))));
        assert_eq!(
            heading("!! [=a=]"),
            Ok(("", Heading(1, vec![DontFormat("a")])))
        );
        assert_eq!(
            heading("!! [[a]]"),
            Ok(("", Heading(1, vec![Link("a", "a")])))
        );
        assert_eq!(
            heading("!! a:[[a]]"),
            Ok(("", Heading(1, vec![Text("a:"), Link("a", "a")])))
        );
        assert_eq!(
            try_pmwikis("!! a"),
            Ok(("", vec![Heading(1, vec![Text("a")])]))
        );
        assert_eq!(
            try_pmwikis("!! a:[[a]]"),
            Ok(("", vec![Heading(1, vec![Text("a:"), Link("a", "a")])]))
        );

        assert_eq!(heading("!!!! b"), Ok(("", Heading(3, vec![Text("b")]))));
        assert_eq!(heading("!!!!! c"), Ok(("", Heading(4, vec![Text("c")]))));
        assert_eq!(pmwikis("!!! b"), vec![Heading(2, vec![Text("b")])]);
        assert_eq!(pmwikis("!!!! c"), vec![Heading(3, vec![Text("c")])]);

        assert_eq!(
            try_pmwikis("!! [[a]]//a"),
            Ok(("", vec![Heading(1, vec![Link("a", "a"), Text("//a")])]))
        );
        assert_eq!(try_pmwikis("!! [[http://www.wikipmwiki.org|Pmwiki]] ''Live'' Editor ([[https://github.com/chidea/wasm-pmwiki-live-editor|github]])"), Ok(("", vec![
      Heading(1, vec![
        Link("http://www.wikipmwiki.org", "Pmwiki"),
        Text(" "),
        Italic(vec![Text("Live")]),
        Text(" Editor ("),
        Link("https://github.com/chidea/wasm-pmwiki-live-editor", "github"),
        Text(")"),
    ])])));
    }

    #[test]
    fn link_tests() {
        init();
        use IPmwiki::*;
        assert_eq!(pmwikis("[[a]]"), vec![Line(vec![Link("a", "a")])]);
        assert_eq!(
            pmwikis("[[https://google.com|google]]"),
            vec![Line(vec![Link("https://google.com", "google")])]
        );
        assert_eq!(link("[[a]]"), Ok(("", Link("a", "a"))));
        assert_eq!(pmwikis("[a]"), vec![Line(vec![Text("[a]")])]);
        assert_eq!(link("[[link|label]]"), Ok(("", Link("link", "label"))));
        assert_eq!(
            link("[[https://google.com|google]]"),
            Ok(("", Link("https://google.com", "google")))
        );
    }

    #[test]
    fn table_tests() {
        init();
        use IPmwiki::*;
        assert_eq!(
            table_header_row("||!a||!||!c||"),
            Ok((
                "",
                TableHeaderRow(vec![
                    TableHeaderCell(vec![Text("a")]),
                    TableHeaderCell(vec![]),
                    TableHeaderCell(vec![Text("c")]),
                ])
            ))
        );
        assert_eq!(
            table("||!a||!||!c||"),
            Ok((
                "",
                Table(vec![TableHeaderRow(vec![
                    TableHeaderCell(vec![Text("a")]),
                    TableHeaderCell(vec![]),
                    TableHeaderCell(vec![Text("c")]),
                ])])
            ))
        );
        // assert_eq!(table_cell_row("|a|b|c|\n"), Ok(("", TableRow(vec![
        //   TableCell(vec![Text("a")]),
        //   TableCell(vec![Text("b")]),
        //   TableCell(vec![Text("c")]),
        // ]))));

        assert_eq!(
            table(
                "||!||!table||!header|| 
||a||table||row|| 
||b||[= // don't format // =]||[= ** me ** =]||
||c||||empty||"
            ),
            Ok((
                "",
                Table(vec![
                    TableHeaderRow(vec![
                        TableHeaderCell(vec![]),
                        TableHeaderCell(vec![Text("table")]),
                        TableHeaderCell(vec![Text("header")]),
                    ]),
                    TableRow(vec![
                        TableCell(vec![Text("a")]),
                        TableCell(vec![Text("table")]),
                        TableCell(vec![Text("row")]),
                    ]),
                    TableRow(vec![
                        TableCell(vec![Text("b")]),
                        TableCell(vec![DontFormat(" // don't format // ")]),
                        TableCell(vec![DontFormat(" ** me ** ")]),
                    ]),
                    TableRow(vec![
                        TableCell(vec![Text("c")]),
                        TableCell(vec![]),
                        TableCell(vec![Text("empty")]),
                    ]),
                ])
            ))
        );
        assert_eq!(
            try_pmwikis(
                "||!||!a||!b||
||0||1||2||
||3||4||5||"
            ),
            Ok((
                "",
                vec![Table(vec![
                    TableHeaderRow(vec![
                        TableHeaderCell(vec![]),
                        TableHeaderCell(vec![Text("a")]),
                        TableHeaderCell(vec![Text("b")])
                    ]),
                    TableRow(vec![
                        TableCell(vec![Text("0")]),
                        TableCell(vec![Text("1")]),
                        TableCell(vec![Text("2")])
                    ]),
                    TableRow(vec![
                        TableCell(vec![Text("3")]),
                        TableCell(vec![Text("4")]),
                        TableCell(vec![Text("5")])
                    ])
                ])]
            ))
        );
    }

    #[test]
    fn image_tests() {
        init();
        use IPmwiki::*;
        assert_eq!(image("[{a.jpg}]"), Ok(("", Image("a.jpg", ""))));
        assert_eq!(image("[{a.jpg|label}]"), Ok(("", Image("a.jpg", "label"))));
        assert_eq!(pmwikis("[{a.jpg}]"), vec![Line(vec![Image("a.jpg", "")])]);
        assert_eq!(
            pmwikis("[{a.jpg|label}]"),
            vec![Line(vec![Image("a.jpg", "label")])]
        );

        assert_eq!(
            pmwikis("[{a.jpg|[[label]]}]"),
            vec![Line(vec![Image("a.jpg", "[[label]]")])]
        );
    }
    #[test]
    fn other_tests() {
        init();
        use IPmwiki::*;
        assert_eq!(try_pmwikis(""), Ok(("", vec![Line(vec![])])));
        assert_eq!(try_pmwikis("----"), Ok(("", vec![HorizontalLine])));
        assert_eq!(
            try_pmwikis("-----"),
            Ok(("", vec![Line(vec![Text("-----")])]))
        );
        assert_eq!(pmwikis("----"), vec![HorizontalLine]);
        // assert_eq!(pmwikis("----a"), vec![Line(vec![Text("----a")])]);
        assert_eq!(
            pmwikis("----\na"),
            vec![HorizontalLine, Line(vec![Text("a")])]
        );
        // assert_eq!(try_pmwikis("a\n----\nb"), Ok(("", vec![Line(vec![Text("a\n")]), HorizontalLine, Line(vec![Text("b")])])));
        // //     assert_eq!(pmwikis("{{a.jpg|b}}"), vec![Image("a.jpg", "b")]);
        //     assert_eq!(dont_format("{{{
        // == [[no]]:\n//**don't** format//
        // }}}"), Ok(("", DontFormat("\n== [[no]]:\n//**don't** format//"))));
    }

    //   // #[cfg(feature="extended")]
    //   // #[test]
    //   // fn extended_tests() { init();
    //   //   // assert_eq!(pmwikis("[{a|b|c}]"), vec![LinkButton("a", "b", "c")]);
    //   //   assert_eq!(link_button("[{a|b|c}]"), LinkButton("a", "b", "c"));
    //   // }
    #[test]
    fn mixed_tests() {
        init();
        use IPmwiki::*;
        assert_eq!(
            try_pmwikis("!! 大"),
            Ok(("", vec![Heading(1, vec![Text("大")])]))
        );
        assert_eq!(
            try_pmwikis("!! a\n!! b\n----"),
            Ok((
                "",
                vec![
                    Heading(1, vec![Text("a")]),
                    Heading(1, vec![Text("b")]),
                    HorizontalLine,
                ]
            ))
        );
        assert_eq!(
            try_pmwikis(
                "!! t

!! A"
            ),
            Ok((
                "",
                vec![
                    Heading(1, vec![Text("t")]),
                    Line(vec![]),
                    Heading(1, vec![Text("A")]),
                ]
            ))
        );
        //     // assert_eq!(try_pmwikis("a[[/|home]]{{a.jpg}}"), Ok(("", vec![
        //     //   Text("a"),
        //     //   Link("/", "home"),
        //     //   Image("a.jpg", ""),
        //     // ])));
    }
}
