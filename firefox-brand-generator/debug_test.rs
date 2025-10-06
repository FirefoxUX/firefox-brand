use regex::Regex;

fn main() {
    let input = "foo\n{{#if GL_ES == false}}\nbar\n{{#endif}}\nfoo";
    let block_if_regex = Regex::new(r"(?m)^\s*\{\{#if\s+([^\s]+)\s*==\s*([^\s]+)\s*\}\}\s*\n(.*?)(?:\s*\{\{#else\}\}\s*\n(.*?))?\s*\{\{#endif\}\}\s*$").unwrap();
    
    if let Some(caps) = block_if_regex.captures(input) {
        println!("Full match: {:?}", caps.get(0).unwrap().as_str());
        println!("Var: {:?}", caps.get(1).unwrap().as_str());
        println!("Value: {:?}", caps.get(2).unwrap().as_str());
        println!("If content: {:?}", caps.get(3).map(|m| m.as_str()));
        println!("Else content: {:?}", caps.get(4).map(|m| m.as_str()));
    }
    
    let result = block_if_regex.replace_all(input, "REPLACEMENT");
    println!("Result: {:?}", result.as_ref());
}
