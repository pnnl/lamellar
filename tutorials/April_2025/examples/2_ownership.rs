fn main(){
    let str1 = String::from("hello");
    let str2 = str1;
    println!(str1);
}


//--- Correct ---//
// fn main(){
//     let str1 = String::from("hello");
//     let str2 = str1; //str1 transfers ownership to str2
//     println!(str2);
// }

//--- Correct (using borrowing)---//
// fn main(){
//     let str1 = String::from("hello");
//     let str2 = &str1;
//     println!(str1);
//     println!(str2);
// }