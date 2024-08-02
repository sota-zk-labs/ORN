use std::collections::{HashMap, HashSet};
use const_format::concatcp;
use fancy_regex::{Captures, Regex};

use crate::const_values::ConstantValue;

const ANY_CHAR_PATTERN: &str = r"([^}]*)";
const CONST_BLOCK_BEGIN: &str =
    "    // This line is used for generating constants DO NOT REMOVE!\n";
const CONST_BLOCK_END: &str = "    // End of generating constants!\n";

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
    let get_const_pattern = Regex::new(&format!(r"({})", get_const_regex(table))).unwrap();
    let comma_pattern = Regex::new(r",+").unwrap();
    Regex::new(&get_import_regex(table))
        .unwrap()
        .replace_all(&file_content, |caps: &Captures| {
            let new_line = caps.get(0).unwrap().as_str();
            let new_line = new_line.chars().filter(|c| !c.is_whitespace()).collect::<String>();
            let new_line = get_const_pattern.replace_all(&new_line, |_: &Captures| {
                ""
            });
            let new_line = comma_pattern.replace_all(&new_line, |_: &Captures| {
                ","
            }).replace(",}", "}").replace("{,", "{");
            if new_line.contains("{}") || new_line.contains("::;") {
                "".to_string()
            } else {
                format!("    use {}\n", new_line[3..new_line.len()].to_string())
            }
        })
        .to_string()
}

pub fn get_import_regex(table: &HashMap<String, ConstantValue>) -> String {
    let any_character = r"[a-zA-Z0-9_:,{}()\s]*";
    format!(r"    use\s+{}({}){};\n",
            any_character,
            get_const_regex(table),
            any_character,
    ).to_string()
}

pub fn get_const_funcs_regex(table: &HashMap<String, ConstantValue>) -> String {
    format!("{}({}){}{}{}",
            r"\s*public fun ",
            get_const_regex(table),
            r"\(\)\s*:\s*(\w+)\s*\{\s*",
            ANY_CHAR_PATTERN,
            r"*\}").to_string()
}

pub fn get_const_regex(table: &HashMap<String, ConstantValue>) -> String {
    let result = table.iter().map(|(k, _)| k).fold("".to_string(), |acc, k| {
        format!("{}({})|", acc, k)
    });
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


    // find all constant usages, remove '()' if it's a function call
    let mut consts = HashSet::<String>::new();
    let mut funcs = vec![];
    let const_regex_func_calls = format!("({}){}", get_const_regex(table), r"(\(\))?");
    result = Regex::new(&const_regex_func_calls)
        .unwrap()
        .replace_all(&result, |caps: &Captures| {
            let name = caps[1].to_string();
            // const not in table, so don't replace
            consts.insert(name.clone());
            // add imported function for removing later
            if caps[0].to_string().ends_with("()")
                && !funcs.contains(&name)
                && !declared_funcs.contains(&name)
            {
                funcs.push(name.clone());
            }
            name
        })
        .to_string();

    result = remove_import(&result, &table);

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
    fn test_gen_consts_sample1() {
        let file_content = include_str!("./test_files/sample1_input.move");
        let refined_content = include_str!("./test_files/sample1_expect.move");
        let output = gen_consts(file_content, &get_constant_values());
        assert_eq!(
            refined_content,
            output,
            "failed"
        );
    }
    #[test]
    fn test_gen_consts_sample2() {
        let file_content = include_str!("./test_files/sample2_input.move");
        let refined_content = include_str!("./test_files/sample2_expect.move");
        let output = gen_consts(file_content, &get_constant_values());
        assert_eq!(
            refined_content,
            output,
            "failed"
        );
    }
}
