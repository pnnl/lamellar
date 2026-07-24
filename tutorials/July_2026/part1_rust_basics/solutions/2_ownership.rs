fn main(){
    let str1 = String::from("hello");
    let str2 = &str1; // borrow instead of moving
    println!("{str1}");
    println!("{str2}");
}
