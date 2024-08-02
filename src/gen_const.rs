use std::collections::HashMap;

use const_format::concatcp;
use fancy_regex::{Captures, Regex};

use crate::const_values::ConstantValue;

const SNAKE_CASE_PATTERN: &str = r"([A-Z][A-Z0-9_]*)";
const ANY_CHAR_PATTERN: &str = r"([^}]*)";
const CONST_FUNC_PATTERN: &str = concatcp!(
    r"\s*public fun ",
    SNAKE_CASE_PATTERN,
    r"\(\)\s*:\s*(\w+)\s*\{\s*",
    ANY_CHAR_PATTERN,
    r"*\}"
);
const CONST_BLOCK_BEGIN: &str =
    "    // This line is used for generating constants DO NOT REMOVE!\n";
const CONST_BLOCK_END: &str = "    // End of generating constants!\n";

fn create_const_block(consts: &Vec<String>, table: &HashMap<String, ConstantValue>) -> String {
    if consts.is_empty() {
        return "".to_string();
    }
    let mut result = CONST_BLOCK_BEGIN.to_string();
    for c in consts {
        let info = table.get(c).unwrap();
        if let Some(comment) = info.comment.clone() {
            result += format!("    // {}\n", comment).as_str();
        }
        result += format!("    const {}: {} = {};\n", c, info.r#type, info.value).as_str();
    }
    result += CONST_BLOCK_END;
    result
}

fn remove_import(file_content: &mut String, const_name: &str) {
    let i = Regex::new(&format!(r"\b{}(?!\s*\()", const_name))
        .unwrap()
        .find(file_content)
        .unwrap();
    if i.is_none() {
        return;
    }
    let i = i.unwrap().start();
    let (mut l, mut r) = (i, i + const_name.len());
    let left_cb = u8::try_from('{').unwrap();
    let right_cb = u8::try_from('}').unwrap();
    let comma = u8::try_from(',').unwrap();
    let colon = u8::try_from(':').unwrap();
    let semi_colon = u8::try_from(';').unwrap();
    let bn = u8::try_from('\n').unwrap();
    let space = u8::try_from(' ').unwrap();
    loop {
        let c = file_content.as_bytes()[l];
        if c == left_cb || c == colon || c == comma {
            if c == colon {
                while file_content.as_bytes()[l] != bn {
                    l -= 1;
                }
            }
            break;
        }
        l -= 1;
    }
    loop {
        let c = file_content.as_bytes()[r];
        if c == right_cb || c == semi_colon || c == comma {
            let cl = file_content.as_bytes()[l];
            if c == comma {
                if cl == comma {
                    r -= 1;
                } else {
                    l += 1;
                    if file_content.as_bytes()[r + 1] == space {
                        r += 1;
                    }
                }
            }
            if c == right_cb {
                if cl == comma {
                    r -= 1;
                } else {
                    r += 1;
                    while file_content.as_bytes()[l] != bn {
                        l -= 1;
                    }
                }
            }
            break;
        }
        r += 1;
    }
    file_content.drain(l..=r);
}
pub fn gen_consts(file_content: &str, table: &HashMap<String, ConstantValue>) -> String {
    // remove constant function declaration
    let mut declared_funcs = vec![];
    let mut result = Regex::new(CONST_FUNC_PATTERN)
        .unwrap()
        .replace_all(file_content, |cap: &Captures| {
            declared_funcs.push(cap[1].to_string());
            ""
        })
        .to_string();

    // find all constant usages, remove '()' if it's a function call
    let mut consts = vec![];
    let mut funcs = vec![];
    result = Regex::new(concatcp!(SNAKE_CASE_PATTERN, r"(\(\))?"))
        .unwrap()
        .replace_all(&result, |caps: &Captures| {
            let name = caps[1].to_string();
            // const not in table, so don't replace
            if !table.contains_key(&name) {
                caps[0].to_string()
            } else {
                if !consts.contains(&name) {
                    consts.push(name.clone());
                }
                // add imported function for removing later
                if caps[0].to_string().ends_with("()")
                    && !funcs.contains(&name)
                    && !declared_funcs.contains(&name)
                {
                    funcs.push(name.clone());
                }
                name
            }
        })
        .to_string();

    // remove imported functions
    for func in funcs {
        remove_import(&mut result, &func);
    }

    // insert constant block into the beginning of the module
    let reg = Regex::new(concatcp!(
        CONST_BLOCK_BEGIN,
        ANY_CHAR_PATTERN,
        CONST_BLOCK_END
    ))
    .unwrap();
    // if constant block was generated before
    if reg.is_match(&result).unwrap() {
        result = reg
            .replace(&result, |_cap: &Captures| {
                create_const_block(&consts, table)
            })
            .to_string();
    } else if !consts.is_empty() {
        let first_left_cb = result.find('{').unwrap();
        result.insert_str(
            first_left_cb + 1,
            format!("\n{}", create_const_block(&consts, table)).as_str(),
        );
    }

    result
}

#[cfg(test)]
mod test {
    use crate::const_values::get_constant_values;
    use crate::gen_const::gen_consts;

    #[test]
    fn test_gen_consts() {
        let file_content = include_str!("./test_files/sample1_input.move");
        let refined_content = include_str!("./test_files/sample1_expect.move");
        assert_eq!(
            refined_content,
            gen_consts(file_content, &get_constant_values()),
            "oke"
        );
    }
}
