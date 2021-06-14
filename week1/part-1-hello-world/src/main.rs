fn main() {
    // println!("Hello, world!");
    let mut str1 = String::from("hello");
    let s1 = &mut str1;
    println!("{}", s1);
    let s2 = &mut str1;
    println!("{}", s2);
}
