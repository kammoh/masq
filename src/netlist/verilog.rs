use std::{fs::File, io::Read, path::PathBuf, cmp, collections::HashMap};

use sv_parser::parse_sv;

fn print_parse_error(origin_path: &PathBuf, origin_pos: &usize) {
    static CHAR_CR: u8 = 0x0d;
    static CHAR_LF: u8 = 0x0a;

    let mut f = File::open(&origin_path).unwrap();
    let mut s = String::new();
    let _ = f.read_to_string(&mut s);

    let mut pos = 0;
    let mut column = 1;
    let mut last_lf = None;
    while pos < s.len() {
        if s.as_bytes()[pos] == CHAR_LF {
            column += 1;
            last_lf = Some(pos);
        }
        pos += 1;

        if *origin_pos == pos {
            let row = if let Some(last_lf) = last_lf {
                pos - last_lf
            } else {
                pos + 1
            };
            let mut next_crlf = pos;
            while next_crlf < s.len() {
                if s.as_bytes()[next_crlf] == CHAR_CR || s.as_bytes()[next_crlf] == CHAR_LF {
                    break;
                }
                next_crlf += 1;
            }

            let column_len = format!("{}", column).len();

            eprint!(" {}:{}:{}\n", origin_path.to_string_lossy(), column, row);

            eprint!("{}|\n", " ".repeat(column_len + 1));

            eprint!("{} |", column);

            let beg = if let Some(last_lf) = last_lf {
                last_lf + 1
            } else {
                0
            };
            eprint!(
                " {}\n",
                String::from_utf8_lossy(&s.as_bytes()[beg..next_crlf])
            );

            eprint!("{}|", " ".repeat(column_len + 1));

            eprint!(
                " {}{}\n",
                " ".repeat(pos - beg),
                "^".repeat(cmp::min(origin_pos + 1, next_crlf) - origin_pos)
            );
        }
    }
}

fn read_netlist() {
    let path = 
    match parse_sv(&path, &HashMap::new(), &[], true, false) {
        Ok((syntax_tree, new_defines)) => {
            println!("  - file_name: {}", escape_str(path.to_str().unwrap()));
            if !opt.full_tree {
                println!("    defs:");
                analyze_defs(&syntax_tree);
            } else {
                println!("    syntax_tree:");
                print_full_tree(&syntax_tree, opt.include_whitespace);
            }
            // update the preprocessor state if desired
            if !opt.separate {
                defines = new_defines;
            }
            // show macro definitions if desired
            if opt.show_macro_defs {
                println!("    macro_defs:");
                show_macro_defs(&defines);
            }
        }
        Err(x) => {
            match x {
                sv_parser::Error::Parse(Some((origin_path, origin_pos))) => {
                    eprintln!("parse failed: {:?}", path);
                    print_parse_error(&origin_path, &origin_pos);
                }
                x => {
                    eprintln!("parse failed: {:?} ({})", path, x);
                    let mut err = x.source();
                    while let Some(x) = err {
                        eprintln!("  Caused by {}", x);
                        err = x.source();
                    }
                }
            }
            exit_code = 1;
        }
    }
}

#[cfg(test)]
mod tests {
    fn test_parse_netlist() {}
}
