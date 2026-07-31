use std::io::{stdin, stdout, Read, Write};
use std::process::exit;
use std::fs;

#[derive(Debug)]
struct EnvVec {
    data: Vec<u8>,
    index: usize
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
    INP
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
                while map.get() != 0 {
                    for t in v.iter() {
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
            Token::INP => println!("{}", map.get())
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
}

fn get_len_token(toks: &Vec<Token>) -> usize {
    let mut len = 0;
    for tok in toks.iter() {
        len += match tok {
            Token::LEFTW(l) => {
                get_len_token(l) + 1
            },
            _ => 1
        }
    }
    len
}

fn parse(command: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let commands: Vec<String> = command.chars()
        .filter(
            |x|
                matches!(x, '>' | '<' | '+' | '-' |
                    '.' | ',' | '[' | ']' | '0'..='9' | '!' |
                '=' | '~' | '{' | '}'))
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
                // println!("ZSX{}", iter.to_string());
                let curr = parse(command_iter.to_string().as_str()).clone();
                command_iter.index += get_len_token(&curr);
                // dbg!(&curr);
                if !curr.contains(&Token::RIGHTW) {
                    eprintln!("[]未闭合!");
                    exit(-1);
                }
                for i in curr {
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
                return tokens
            },
            "0" => Token::NUM(0),
            "1" => Token::NUM(1),
            "2" => Token::NUM(2),
            "3" => Token::NUM(3),
            "4" => Token::NUM(4),
            "5" => Token::NUM(5),
            "6" => Token::NUM(6),
            "7" => Token::NUM(7),
            "8" => Token::NUM(8),
            "9" => Token::NUM(9),
            "=" => Token::SZ,
            "~" => Token::BITNOT,
            "{" => Token::LM,
            "}" => Token::RM,
            "!" => Token::INP,
            _ => Token::NONE
        };
        tokens.push(curr_com);
    }
    tokens
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
            index: 0
        };
        loop {
            let mut inp = String::new();
            print!(">>>");
            stdout().flush().unwrap();
            stdin().read_line(&mut inp).unwrap();
            if inp.trim() == String::from("exit") {
                exit(0);
            }
            run(inp.as_str(), &mut env);
            println!();
        }
    } else {
        let file = &args[1];
        let content = String::new();
        match fs::read_to_string(file) {
            Ok(val) => {
                let mut env = EnvVec{
                    data: vec![0],
                    index: 0
                };
                run(val.as_str(), &mut env);
            },
            Err(e) => {
                eprintln!("错误: {e}");
                exit(-2);
            }
        }
    }
}

fn run(test: &str, env: &mut EnvVec) {
    let ret = parse(test);
    // dbg!(&ret);
    for tok in ret {
        tok.exec(env);
        // dbg!(&env);
    }
}
