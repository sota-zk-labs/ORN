use std::collections::{HashMap, HashSet};

use const_format::concatcp;
use fancy_regex::{Captures, Regex};

use crate::const_values::ConstantValue;

const SNAKE_CASE_PATTERN: &str = r"([A-Z][A-Z0-9]*[_[A-Z0-9]+]+)";
const ANY_CHAR_PATTERN: &str = r"([^}]*)";
const IMPORT_STATEMENT_PATTERN: &str = r"\s*use\s+.*::([^;]+);";
const CONST_BLOCK_BEGIN: &str =
    "    // This line is used for generating constants DO NOT REMOVE!\n";
const CONST_BLOCK_END: &str = "    // End of generating constants!\n\n";

fn create_const_block(consts: &HashSet<String>, table: &HashMap<String, ConstantValue>) -> String {
    if consts.is_empty() {
        return "".to_string();
    }
    let mut consts: Vec<_> = consts.iter().collect();
    consts.sort();

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

fn remove_import(file_content: &str, table: &HashMap<String, ConstantValue>) -> String {
    Regex::new(IMPORT_STATEMENT_PATTERN)
        .unwrap()
        .replace_all(file_content, |cap: &Captures| {
            let mut new_statement = cap[1].trim().to_string();
            if Regex::new(SNAKE_CASE_PATTERN)
                .unwrap()
                .is_match(&new_statement)
                .unwrap()
            {
                if new_statement.starts_with('{') {
                    // Case 1: CONST,
                    new_statement = Regex::new(&format!(r"{}\s*,\s*", SNAKE_CASE_PATTERN))
                        .unwrap()
                        .replace_all(&new_statement, |import_caps: &Captures| {
                            let const_name = import_caps[1].to_string();
                            if table.contains_key(&const_name) {
                                "".to_string()
                            } else {
                                import_caps[0].to_string()
                            }
                        })
                        .to_string();
                    // Case 2: , CONST and CONST
                    new_statement = Regex::new(&format!(r",?\s*{}\s*", SNAKE_CASE_PATTERN))
                        .unwrap()
                        .replace_all(&new_statement, |import_caps: &Captures| {
                            let const_name = import_caps[1].to_string();
                            if table.contains_key(&const_name) {
                                "".to_string()
                            } else {
                                import_caps[0].to_string()
                            }
                        })
                        .to_string();
                } else {
                    new_statement = "".to_string();
                };
                if new_statement.ends_with("{}") {
                    new_statement = "".to_string();
                }
                if new_statement.is_empty() {
                    "".to_string()
                } else {
                    cap[0]
                        .replace(cap[1].to_string().as_str(), new_statement.as_ref())
                        .to_string()
                }
            } else {
                cap[0].to_string()
            }
        })
        .to_string()
}

pub fn get_import_regex(table: &HashMap<String, ConstantValue>) -> String {
    let any_character = r"[a-zA-Z0-9_:,{}()\s]*";
    format!(
        r"    use\s+{}({}){};\n",
        any_character,
        get_const_regex(table),
        any_character,
    )
    .to_string()
}

pub fn get_const_funcs_regex(table: &HashMap<String, ConstantValue>) -> String {
    format!(
        "{}({}){}{}{}",
        r"\s*public fun ",
        get_const_regex(table),
        r"\(\)\s*:\s*(\w+)\s*\{\s*",
        ANY_CHAR_PATTERN,
        r"*\}"
    )
    .to_string()
}

pub fn get_const_regex(table: &HashMap<String, ConstantValue>) -> String {
    let result = table
        .iter()
        .map(|(k, _)| k)
        .fold("".to_string(), |acc, k| format!("{}({})|", acc, k));
    result[0..result.len() - 1].to_string()
}

pub fn gen_consts(file_content: &str, table: &HashMap<String, ConstantValue>) -> String {
    // remove constant function declaration
    let mut declared_funcs = vec![];
    let mut result = Regex::new(&get_const_funcs_regex(table))
        .unwrap()
        .replace_all(file_content, |cap: &Captures| {
            declared_funcs.push(cap[1].to_string());
            ""
        })
        .to_string();

    // remove constants block if it was generated before
    result = Regex::new(concatcp!(
        CONST_BLOCK_BEGIN,
        ANY_CHAR_PATTERN,
        CONST_BLOCK_END
    ))
    .unwrap()
    .replace_all(&result, |_: &Captures| format!("{}{}", CONST_BLOCK_BEGIN, CONST_BLOCK_END))
    .to_string();

    // remove '()' if it's a constant function call
    let mut consts = HashSet::<String>::new();
    let const_regex_func_calls = format!(r"{}(\(\))?", SNAKE_CASE_PATTERN);
    result = Regex::new(&const_regex_func_calls)
        .unwrap()
        .replace_all(&result, |caps: &Captures| {
            let name = caps[1].to_string();
            if table.contains_key(&name) {
                consts.insert(name.clone());
                name
            } else {
                caps[0].to_string()
            }
        })
        .to_string();

    result = remove_import(&result, table);

    // insert constants block
    if !consts.is_empty() {
        // replace old constants block with new block
        if result.contains(format!("{}{}", CONST_BLOCK_BEGIN, CONST_BLOCK_END).as_str()) {
            result = result.replace(format!("{}{}", CONST_BLOCK_BEGIN, CONST_BLOCK_END).as_str(), create_const_block(&consts, table).as_str());
        } else {
            // insert into the beginning of the module
            let mut first_left_cb = result.find('{').unwrap();
            while result.as_bytes()[first_left_cb] != u8::try_from('\n').unwrap() {
                first_left_cb += 1;
            }
            result.insert_str(
                first_left_cb + 1,
                create_const_block(&consts, table).as_str(),
            );
        }
    }

    result
}

#[cfg(test)]
mod test {
    use crate::const_values::get_constant_values;
    use crate::gen_const::gen_consts;

    #[test]
    fn test_gen_consts_sample1() {
        let file_content = include_str!("./test_files/sample1_input.move");
        let refined_content = include_str!("./test_files/sample1_expect.move");
        let output = gen_consts(file_content, &get_constant_values());
        assert_eq!(refined_content, output, "failed");
    }
    #[test]
    fn test_gen_consts_sample2() {
        let file_content = include_str!("./test_files/sample2_input.move");
        let refined_content = include_str!("./test_files/sample2_expect.move");
        let output = gen_consts(file_content, &get_constant_values());
        assert_eq!(refined_content, output, "failed");
    }
}
