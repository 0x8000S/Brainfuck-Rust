use std::collections::HashMap;
use std::io::{stdin, stdout, Read, Write};
use std::process::exit;
use std::fs;

#[derive(Debug)]
struct EnvVec {
    data: Vec<u8>,
    index: usize,
    stack: Vec<u8>,
    func: HashMap<String, Vec<Token>>
}

impl EnvVec {
    fn check(&mut self) {
        if self.index + 1 > self.data.len() {
            for _ in 0..self.index+1-self.data.len() {
                self.data.push(0);
            }
        }
    }
    fn left(&mut self) {
        self.check();
        if self.index > 0 {
            self.index -= 1;
        }
    }
    fn right(&mut self) {
        self.index += 1;
        self.check();

    }
    fn plus(&mut self) {
        self.check();
        self.data[self.index] = self.data[self.index].wrapping_add(1);
    }
    fn sub(&mut self) {
        self.check();
        self.data[self.index] = self.data[self.index].wrapping_sub(1);
    }
    fn get(&mut self) -> u8 {
        self.check();
        self.data[self.index]
    }
    fn set(&mut self, data: u8) {
        self.check();
        self.data[self.index] = data;
    }
    fn left_move(&mut self) {
        self.data[self.index] = self.data[self.index].wrapping_mul(2);
    }
    fn right_move(&mut self) {
        self.data[self.index] = self.data[self.index].wrapping_div(2);
    }
    fn push_stack(&mut self) {
        self.stack.push(self.data[self.index])
    }
    fn pop_stack(&mut self) {
        self.data[self.index] = self.stack.pop().unwrap_or_else(|| 0);
    }
    fn reg_func(&mut self, name: String, tokens: Vec<Token>) {
        self.func.insert(name, tokens);
    }
    fn get_func(&self, name: &String) -> Vec<Token> {
        let ret = self.func.get(name);
        match ret { 
            Some(val) => val.clone(),
            None => vec![]
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
enum Token {
    LEFT,
    RIGHT,
    PLUS,
    SUB,
    PRT,
    READ,
    LEFTW(Vec<Token>),
    RIGHTW,
    NONE,
    NUM(u8),
    SZ,
    BITNOT,
    LM,
    RM,
    INP,
    PUSH,
    POP,
    LEFTPAR,
    RIGHTPAR,
    FUNC(String, Vec<Token>),
    CALLFUNC(String),
    IMPORT(String),
    ENDIMPORT
}
impl Token {
    fn exec(&self, map: &mut EnvVec) {
        match self {
            Token::LEFT => map.left(),
            Token::RIGHT => map.right(),
            Token::PLUS => map.plus(),
            Token::SUB => map.sub(),
            Token::PRT => {
                print!("{}", String::from_utf8_lossy(&[map.get()]));
                stdout().flush().unwrap();

            },
            Token::READ => {
                let mut buf = [0u8; 1];
                match stdin().read_exact(&mut buf) {
                    Ok(_) => map.set(buf[0]),
                    Err(_) => map.set(0),
                }
            },
            Token::LEFTW(v) => {
                // println!("Into while");
                let mut stop_num = 0;
                if v.len() >= 1 {
                    match v[0] {
                        Token::NUM(val) => stop_num = val,
                        _ => ()
                    }
                }
                while map.get() != stop_num {
                    for (i, t) in v.iter().enumerate() {
                        if i == 0 {
                            match t {
                                Token::NUM(v) => continue,
                                _ => ()
                            }
                        }
                        t.exec(&mut *map);
                        // dbg!(&map);
                    }
                }
                // println!("Exit while");
            },
            Token::RIGHTW => (),
            Token::NONE => (),
            Token::NUM(n) => map.set(*n),
            Token::SZ => map.set(0),
            Token::BITNOT => {
                let value = map.get();
                map.set(!value);
            },
            Token::LM => map.left_move(),
            Token::RM => map.right_move(),
            Token::INP => println!("{}", map.get()),
            Token::PUSH => map.push_stack(),
            Token::POP => map.pop_stack(),
            Token::LEFTPAR => (),
            Token::RIGHTPAR => (),
            Token::FUNC(name, tokens) => map.reg_func(name.clone(), tokens.clone()),
            Token::CALLFUNC(name) => {
                for t in map.get_func(name) {
                    t.exec(map);
                }
            },
            Token::IMPORT(mo) => {
                for t in parse(read_code_from_file(mo).as_str()).0 {
                    t.exec(map);
                }
            },
            Token::ENDIMPORT => ()
        }
    }
}
#[derive(Debug)]
struct CommandContent {
    data: Vec<String>,
    index: usize
}
impl CommandContent {
    fn next(&mut self) -> Option<String> {
        if self.index+1 > self.data.len() {
            return None
        }
        let ret = Some(self.data[self.index].clone());
        self.index += 1;
        ret
    }
    fn to_string(&self) -> String {
        let mut ret = String::new();
        for i in &self.data[self.index..] {
            ret.push_str(i.clone().as_str())
        }
        ret
    }
    fn peek(&self) -> Option<String> {
        match self.data.get(self.index) {
            Some(val) => Some(val.clone()),
            None => None
        }
    }
}

fn is_num(num: &str) -> bool {
    match num.parse::<u64>() {
        Ok(v) => true,
        Err(_) => false
    }
}

fn parse_to_token(command: &str) -> Token {
    match command {
        "<" => Token::LEFT,
        ">" => Token::RIGHT,
        "+" => Token::PLUS,
        "-" => Token::SUB,
        "." => Token::PRT,
        "," => Token::READ,
        "[" => Token::LEFTW(vec![]),
        "]" => Token::RIGHTW,
        "=" => Token::SZ,
        "~" => Token::BITNOT,
        "{" => Token::LM,
        "}" => Token::RM,
        "!" => Token::INP,
        ":" => Token::PUSH,
        ";" => Token::POP,
        "(" => Token::LEFTPAR,
        ")" => Token::RIGHTPAR,
        "`" => Token::FUNC(String::new(), vec![]),
        "@" => Token::IMPORT(String::new()),
        "|" => Token::ENDIMPORT,
        _ => {
            if is_num(&command) {
                Token::NUM(0)
            }else {
                Token::CALLFUNC(String::new())
            }
        }
    }
}

fn parse(command: &str) -> (Vec<Token>, usize) {
    let mut tokens: Vec<Token> = Vec::new();
    let commands: Vec<String> = command.chars()
        .map(|x| x.to_string())
        .collect();
    // dbg!(&commands);
    let mut command_iter = CommandContent {
        data: commands.clone(),
        index: 0
    };
    // let mut command_iter = Rc::new(RefCell::new(commands.iter()));

    loop {
        let z = command_iter.next();
        // dbg!(&command_iter);
        let t;
        if let Some(x) = z {
            t = x;
        } else {
            // println!("exit main parse loop");
            break;
        }
        // println!("{t}");
        let curr_com = match t.as_str() {
            "<" => Token::LEFT,
            ">" => Token::RIGHT,
            "+" => Token::PLUS,
            "-" => Token::SUB,
            "." => Token::PRT,
            "," => Token::READ,
            "[" => {
                let mut p_command = Vec::new();
                // dbg!(&iter);
                // dbg!(&command_iter);
                // println!("ZSX{}", iter.to_string());
                let curr = parse(command_iter.to_string().as_str()).clone();
                // command_iter.index += get_len_token(&curr.0);
                command_iter.index += curr.1;
                // dbg!(&curr);
                if !curr.0.contains(&Token::RIGHTW) {
                    dbg!(&curr);
                    eprintln!("[]未闭合!");
                    exit(-1);
                }
                for i in curr.0 {
                    p_command.push(i)
                }
                // dbg!(&iter);
                // println!("闭合!");
                // dbg!(&command_iter);
                Token::LEFTW(p_command)
            },
            "]" => {
                tokens.push(Token::RIGHTW);
                // dbg!(&tokens);
                return (tokens, command_iter.index)
            },
            "=" => Token::SZ,
            "~" => Token::BITNOT,
            "{" => Token::LM,
            "}" => Token::RM,
            "!" => Token::INP,
            ":" => Token::PUSH,
            ";" => Token::POP,
            "(" => Token::LEFTPAR,
            ")" => {
                tokens.push(Token::RIGHTPAR);
                return (tokens, command_iter.index);
            }
            "`" => {
                let mut func_name = String::new();
                let mut func_body = Vec::new();
                loop {
                    match command_iter.next() {
                        Some(val) => {
                            if parse_to_token(val.as_str()) == Token::LEFTPAR {
                                let ret = parse(command_iter.to_string().as_str());
                                // dbg!(&ret);
                                // command_iter.index += get_len_token(&ret.0);
                                command_iter.index += ret.1;
                                if !ret.0.contains(&Token::RIGHTPAR) {
                                    eprintln!("未找到闭合的()!");
                                    exit(-3);
                                }
                                for t in ret.0 {
                                    func_body.push(t);
                                }
                                break;
                            } else {
                                func_name.push_str(val.as_str());
                            }
                        },
                        None => break
                    }
                }
                Token::FUNC(func_name, func_body)
            },
            "@" => {
                let mut file = String::new();
                let mut close = false;
                loop {
                    match command_iter.peek() {
                        Some(val) => match parse_to_token(val.as_str()) {
                            Token::ENDIMPORT => {
                                close = true;
                                break
                            },
                            _ => ()
                        },
                        None => break
                    }
                    match command_iter.next() {
                        Some(val) => {
                            file.push_str(val.as_str());
                        },
                        None => break
                    }
                }
                if !close {
                    eprintln!("@未闭合|!");
                    exit(-5);
                }
                Token::IMPORT(file)
            },
            "|" => Token::ENDIMPORT,
            _ => {
                if is_num(&t) {
                    let mut num = String::new();
                    num.push_str(t.as_str());
                    loop {
                        match command_iter.peek() {
                            Some(val) => match parse_to_token(val.as_str()) {
                                Token::NUM(_) => (),
                                _ => break
                            },
                            None => break
                        }
                        match command_iter.next() {
                            Some(val) => {
                                num.push_str(val.as_str());
                            },
                            None => break
                        }
                    }
                    // dbg!(&command_iter);
                    Token::NUM((num.parse::<u64>().unwrap() % 256) as u8)
                }else {
                    let mut call_func_name = String::new();
                    call_func_name.push_str(t.as_str());
                    loop {
                        match command_iter.peek() {
                            Some(val) => match parse_to_token(val.as_str()) {
                                Token::CALLFUNC(_) => (),
                                _ => break
                            },
                            None => break
                        }
                        match command_iter.next() {
                            Some(val) => {
                                // match parse(val.as_str())[0] {
                                //     Token::CALLFUNC(_) => (),
                                //     _ => {
                                //         command_iter.index -= 1;
                                //         continue
                                //     }
                                // }
                                call_func_name.push_str(val.as_str());
                            },
                            None => break
                        }
                    }
                    Token::CALLFUNC(call_func_name)
                }
            }
        };
        tokens.push(curr_com);
    }
    (tokens, command_iter.index)
}

fn read_code_from_file(file: &String) -> String {
    match fs::read_to_string(file) {
        Ok(val) => {
            let mut content = String::new();
            for l in val.lines() {
                let lw = match l.find("#") {
                        Some(v) => l[..v].to_string(),
                        None => l.to_string()
                    };
                content.push_str(lw.trim())
                }
            content
        },
        Err(e) => {
            eprintln!("错误: {e}");
            exit(-2);
        }
    }
}

fn main() {
    // let a = vec![Token::RIGHTW, Token::LEFTW(vec![Token::RIGHT, Token::RIGHTW]), Token::READ];
    // println!("{}", a.len());
    // println!("{}", get_len_token(&a));
    // let test = "++++++++++[>+++++++<-]>.";
    let args:Vec<_> = std::env::args().collect();
    if args.len() == 1 {
        let mut env = EnvVec{
            data: vec![0],
            index: 0,
            stack: vec![],
            func: HashMap::new()
        };
        loop {
            let mut inp = String::new();
            print!(">>>");
            stdout().flush().unwrap();
            stdin().read_line(&mut inp).unwrap();
            if inp.trim() == String::from("exit") {
                exit(0);
            }
            run(inp.trim(), &mut env);
            println!();
        }
    } else {
        let file = &args[1];
        let mut env = EnvVec{
            data: vec![0],
            index: 0,
            stack: vec![],
            func: HashMap::new()
        };
        let content = read_code_from_file(file);
        run(content.as_str(), &mut env);
    }
    // let test = "`add(67)add.";
    // // 输出了A-Z
    // let mut env = EnvVec {
    //     data: vec![0],
    //     index: 0,
    //     stack: vec![],
    //     func: HashMap::new()
    // };
    // run(test, &mut env);
}

fn run(test: &str, env: &mut EnvVec) {
    // println!("{test}");
    let ret = parse(test);
    // dbg!(&ret.0);
    for tok in ret.0 {
        tok.exec(env);
        // dbg!(&env);
    }
}
