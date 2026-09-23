use jsonst::parse_file;

fn main() {
    println!("Running JSONst");

    let _ = parse_file("./samples/list.json");

}
