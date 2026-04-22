/// 递归下降四则运算解析器
/// 支持 + - * / 和括号，支持浮点数
/// 语法：
///   expr   = term   (('+' | '-') term)*
///   term   = factor (('*' | '/') factor)*
///   factor = '(' expr ')' | ['-'] number

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Parser {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn skip_whitespace(&mut self) {
        while self.peek().map(|c| c.is_ascii_whitespace()).unwrap_or(false) {
            self.pos += 1;
        }
    }

    fn consume(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    /// expr = term (('+' | '-') term)*
    fn parse_expr(&mut self) -> Option<f64> {
        let mut left = self.parse_term()?;
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('+') => {
                    self.consume();
                    let right = self.parse_term()?;
                    left += right;
                }
                Some('-') => {
                    self.consume();
                    let right = self.parse_term()?;
                    left -= right;
                }
                _ => break,
            }
        }
        Some(left)
    }

    /// term = factor (('*' | '/') factor)*
    fn parse_term(&mut self) -> Option<f64> {
        let mut left = self.parse_factor()?;
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('*') => {
                    self.consume();
                    let right = self.parse_factor()?;
                    left *= right;
                }
                Some('/') => {
                    self.consume();
                    let right = self.parse_factor()?;
                    if right == 0.0 {
                        return None; // 除以零
                    }
                    left /= right;
                }
                _ => break,
            }
        }
        Some(left)
    }

    /// factor = '(' expr ')' | ['-'] number
    fn parse_factor(&mut self) -> Option<f64> {
        self.skip_whitespace();
        match self.peek()? {
            '(' => {
                self.consume(); // 消耗 '('
                let val = self.parse_expr()?;
                self.skip_whitespace();
                if self.consume() != Some(')') {
                    return None; // 括号不匹配
                }
                Some(val)
            }
            '-' => {
                self.consume();
                let val = self.parse_factor()?;
                Some(-val)
            }
            c if c.is_ascii_digit() || c == '.' => self.parse_number(),
            _ => None,
        }
    }

    /// 解析浮点数字面量（含科学计数法）
    fn parse_number(&mut self) -> Option<f64> {
        let start = self.pos;
        while self.peek().map(|c| c.is_ascii_digit() || c == '.').unwrap_or(false) {
            self.pos += 1;
        }
        if self.peek().map(|c| c == 'e' || c == 'E').unwrap_or(false) {
            self.pos += 1;
            if self.peek().map(|c| c == '+' || c == '-').unwrap_or(false) {
                self.pos += 1;
            }
            while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                self.pos += 1;
            }
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse::<f64>().ok()
    }
}

/// 判断输入字符串是否像一个数学表达式：
/// 必须包含至少一个运算符或括号，且只含合法字符
fn looks_like_expr(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    // 减号：只有位于数字/右括号之后才算二元运算符
    let has_operator = s.contains('+')
        || s.contains('*')
        || s.contains('/')
        || s.chars().enumerate().any(|(i, c)| {
            c == '-' && i > 0 && {
                let prev = s.chars().nth(i - 1).unwrap_or(' ');
                prev.is_ascii_digit() || prev == ')' || prev == '.'
            }
        });
    if !has_operator {
        return false;
    }
    // 只允许数字、小数点、四则运算符、括号、空白、e/E（科学计数法）
    s.chars().all(|c| {
        c.is_ascii_digit()
            || c == '.'
            || c == '+'
            || c == '-'
            || c == '*'
            || c == '/'
            || c == '('
            || c == ')'
            || c == 'e'
            || c == 'E'
            || c.is_ascii_whitespace()
    })
}

/// 将 f64 结果格式化：整数去掉小数点，浮点最多保留 6 位小数并去除末尾零
fn format_result(val: f64) -> String {
    if val.fract() == 0.0 && val.abs() < 1e15 {
        return format!("{}", val as i64);
    }
    let s = format!("{:.6}", val);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

/// Tauri command：对输入字符串求值，返回计算结果字符串，或 None
#[tauri::command]
pub fn eval_expr(expr: String) -> Option<String> {
    let trimmed = expr.trim();
    if !looks_like_expr(trimmed) {
        return None;
    }
    let mut parser = Parser::new(trimmed);
    let result = parser.parse_expr()?;

    // 确保整个输入都被消耗，排除尾部非法字符
    parser.skip_whitespace();
    if parser.pos != parser.chars.len() {
        return None;
    }

    if result.is_nan() || result.is_infinite() {
        return None;
    }

    Some(format_result(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(eval_expr("1+1".into()), Some("2".into()));
        assert_eq!(eval_expr("(3+4)*2".into()), Some("14".into()));
        assert_eq!(eval_expr("100/4".into()), Some("25".into()));
        assert_eq!(eval_expr("3.14*2".into()), Some("6.28".into()));
    }

    #[test]
    fn test_non_expressions() {
        assert_eq!(eval_expr("firefox".into()), None);
        assert_eq!(eval_expr("hello world".into()), None);
        assert_eq!(eval_expr("".into()), None);
    }

    #[test]
    fn test_division_by_zero() {
        assert_eq!(eval_expr("1/0".into()), None);
    }

    #[test]
    fn test_float_result() {
        assert_eq!(eval_expr("1.5+1.5".into()), Some("3".into()));
        assert_eq!(eval_expr("1/3".into()), Some("0.333333".into()));
    }

    #[test]
    fn test_unary_minus() {
        assert_eq!(eval_expr("10+-3".into()), Some("7".into()));
        assert_eq!(eval_expr("-5*2".into()), Some("-10".into()));
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(eval_expr("3 + 4 * 2".into()), Some("11".into()));
    }
}
