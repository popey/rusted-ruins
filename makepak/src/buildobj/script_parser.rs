
use std::str::FromStr;
use nom::{space, line_ending};
use nom::types::CompleteStr;
use common::hashmap::HashMap;

use common::script::*;
use crate::error::PakCompileError;
use super::expr_parser::*;

named!(end_line<CompleteStr, ()>,
    do_parse!(
        opt!(space) >>
        line_ending >>
        (())
    )
);

#[test]
fn end_line_test() {
    assert_eq!(end_line(CompleteStr("   \naabb")), Ok((CompleteStr("aabb"), ())));
}

macro_rules! array(
    ($i:expr, $submac:ident!( $($args:tt)* )) => ({
        ws!($i, delimited!(
            char!('['),
            separated_list!(
                char!(','),
                ws!( $submac!($($args)*) )),
            char!(']')))
    });
    ($i:expr, $f:expr) => (
        array!($i, call!($f));
    );
);

named!(section_start<CompleteStr, String>,
    do_parse!(
        tag!("---") >>
        space >>
        s: id >>
        end_line >>
        (s)
    )
);

#[test]
fn section_start_test() {
    assert_eq!(section_start(
        CompleteStr("---  section_name \n")), Ok((CompleteStr(""), "section_name".to_string())));
}

named!(jump_instruction<CompleteStr, Instruction>,
    do_parse!(
        ws!(tag!("jump")) >>
        s: delimited!(tag!("("), ws!(id), tag!(")")) >>
        end_line >>
        (Instruction::Jump(s))
    )
);

named!(jump_if_instruction<CompleteStr, Instruction>,
    do_parse!(
        ws!(tag!("jump_if")) >>
        char!('(') >>
        s: ws!(id) >>
        char!(',') >>
        e: ws!(expr) >>
        char!(')') >>
        end_line >>
        (Instruction::JumpIf(s, e))
    )
);

#[test]
fn jump_instruction_test() {
    assert_eq!(
        jump_instruction(CompleteStr(" jump ( other_section ) \n")),
        Ok((CompleteStr(""), Instruction::Jump("other_section".to_owned()))));
    assert_eq!(
        jump_if_instruction(CompleteStr("jump_if(has-key, has_item(key))\n")),
        Ok((CompleteStr(""), Instruction::JumpIf(
            "has-key".to_owned(), Expr::HasItem("key".to_owned())))));
}

named!(special_instruction<CompleteStr, Instruction>,
    do_parse!(
        ws!(tag!("special")) >>
        s: map_res!(delimited!(tag!("("), ws!(symbol), tag!(")")), FromStr::from_str) >>
        end_line >>
        (Instruction::Special(s))
    )
);

#[test]
fn special_instruction_test() {
    assert_eq!(special_instruction(
        CompleteStr("special(shop_buy)\n")),
        Ok((CompleteStr(""), Instruction::Special(SpecialInstruction::ShopBuy))));
    assert_eq!(special_instruction(
        CompleteStr("special(shop_sell)\n")),
        Ok((CompleteStr(""), Instruction::Special(SpecialInstruction::ShopSell))));
}

named!(talk_instruction<CompleteStr, Instruction>,
    do_parse!(
        ws!(tag!("talk")) >>
        text_id: delimited!(char!('('), ws!(id), char!(')')) >>
        end_line >>
        (Instruction::Talk(text_id, Vec::new()))
    )
);

named!(talk_instruction_with_choices<CompleteStr, Instruction>,
    do_parse!(
        ws!(tag!("talk")) >>
        char!('(') >>
        text_id: ws!(id) >>
        char!(',') >>
        choices: array!(delimited!(
            char!('('), separated_pair!(ws!(id), char!(','), ws!(id)), char!(')') )) >>
        char!(')') >>
        end_line >>
        (Instruction::Talk(text_id, choices))
    )
);

named!(gset_instruction<CompleteStr, Instruction>,
    do_parse!(
        ws!(tag!("gset")) >>
        char!('(') >>
        var_name: ws!(id) >>
        char!(',') >>
        value: ws!(expr) >>
        char!(')') >>
        end_line >>
        (Instruction::GSet(var_name, value))
    )
);

named!(receive_money_instruction<CompleteStr, Instruction>,
    do_parse!(
        ws!(tag!("receive_money")) >>
        e: delimited!(char!('('), ws!(expr), char!(')')) >>
        end_line >>
        (Instruction::ReceiveMoney(e))
    )
);

named!(remove_item_instruction<CompleteStr, Instruction>,
    do_parse!(
        ws!(tag!("remove_item")) >>
        item_id: delimited!(char!('('), ws!(id), char!(')')) >>
        end_line >>
        (Instruction::RemoveItem(item_id))
    )
);

#[test]
fn talk_instruction_test() {
    let result = Instruction::Talk(
        "text-id".to_owned(),
        vec![("a".to_owned(), "b".to_owned()), ("c".to_owned(), "d".to_owned())]);
    assert_eq!(
        talk_instruction_with_choices(CompleteStr("talk(text-id, [(a, b), (c, d)])\n")),
        Ok((CompleteStr(""), result)));
}

named!(instruction<CompleteStr, Instruction>,
    alt!(
        jump_instruction |
        jump_if_instruction |
        talk_instruction_with_choices |
        talk_instruction |
        gset_instruction |
        receive_money_instruction |
        remove_item_instruction |
        special_instruction
    )
);

named!(section<CompleteStr, (String, Vec<Instruction>)>,
    do_parse!(
        section: section_start >>
        instructions: many0!(instruction) >>
        (section.to_string(), instructions)
    )
);

named!(sections<CompleteStr, HashMap<String, Vec<Instruction>>>,
    exact!(fold_many0!(
        section,
        HashMap::default(),
        | mut s: HashMap<String, Vec<Instruction>>, section: (String, Vec<Instruction>) | {
            s.insert(section.0, section.1);
            s
        }))
);

pub fn parse(input: &str) -> Result<Script, PakCompileError> {
    match sections(CompleteStr(input)) {
        Ok(o) => {
            Ok(Script::from_map(o.1))
        }
        Err(e) => {
            Err(PakCompileError::ScriptParseError {
                description: e.to_string()
            })
        }
    }
}

#[test]
fn parse_test() {
    let script = r#"--- test_section0
talk(textid0)
special(shop_buy)
jump(test_section1)
--- test_section1
talk(textid1,
     [(aaa, bbb), (ccc, ddd)])
"#;
    let mut result = HashMap::default();

    result.insert(
        "test_section0".to_owned(),
        vec![
            Instruction::Talk("textid0".to_owned(), vec![]),
            Instruction::Special(SpecialInstruction::ShopBuy),
            Instruction::Jump("test_section1".to_owned())
        ]);
    result.insert(
        "test_section1".to_owned(),
        vec![
            Instruction::Talk(
                "textid1".to_owned(),
                vec![("aaa".to_owned(), "bbb".to_owned()), ("ccc".to_owned(), "ddd".to_owned())]),
        ]);

    assert_eq!(sections(CompleteStr(script)), Ok((CompleteStr(""), result)))
}

